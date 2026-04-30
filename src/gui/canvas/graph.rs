//! Force-directed graph canvas renderer.

use std::collections::HashMap;
use std::time::Instant;

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

use crate::gui::message::{GraphEdge, GraphNode, Message};
use crate::gui::theme::{colors, is_light_theme};

#[derive(Debug, Clone)]
pub struct PhysicsSimulation {
    pub alpha: f32,
    pub alpha_decay: f32,
    pub alpha_min: f32,
    pub velocity_decay: f32,
    pub link_distance: f32,
    pub link_strength: f32,
    pub charge_strength: f32,
    pub center_strength: f32,
}

impl Default for PhysicsSimulation {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            alpha_decay: 0.0228,
            alpha_min: 0.001,
            velocity_decay: 0.4,
            link_distance: 150.0,
            link_strength: 0.12,
            charge_strength: -600.0,
            center_strength: 0.02,
        }
    }
}

impl PhysicsSimulation {
    pub fn is_active(&self) -> bool {
        self.alpha > self.alpha_min
    }

    pub fn tick(&mut self, nodes: &mut [GraphNode], edges: &[GraphEdge]) {
        if self.alpha <= self.alpha_min || nodes.is_empty() {
            return;
        }

        let mut forces = vec![Vector::new(0.0, 0.0); nodes.len()];

        // Charge (repulsion)
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let dx = nodes[j].x - nodes[i].x;
                let dy = nodes[j].y - nodes[i].y;
                let dist_sq = (dx * dx + dy * dy).max(25.0);
                let dist = dist_sq.sqrt().max(1.0);
                let magnitude = self.charge_strength.abs() * self.alpha / dist_sq;
                let fx = dx / dist * magnitude;
                let fy = dy / dist * magnitude;
                forces[i].x -= fx;
                forces[i].y -= fy;
                forces[j].x += fx;
                forces[j].y += fy;
            }
        }

        // Link (springs)
        let index = node_index_map(nodes);
        for edge in edges {
            if let (Some(&a), Some(&b)) = (index.get(&edge.from), index.get(&edge.to)) {
                let dx = nodes[b].x - nodes[a].x;
                let dy = nodes[b].y - nodes[a].y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let diff = dist - self.link_distance;
                let force = diff * self.link_strength * self.alpha;
                let fx = dx / dist * force;
                let fy = dy / dist * force;
                forces[a].x += fx;
                forces[a].y += fy;
                forces[b].x -= fx;
                forces[b].y -= fy;
            }
        }

        // Centering force
        for (i, node) in nodes.iter().enumerate() {
            forces[i].x += -node.x * self.center_strength * self.alpha;
            forces[i].y += -node.y * self.center_strength * self.alpha;
        }

        // Integrate
        for (i, node) in nodes.iter_mut().enumerate() {
            if node.pinned {
                continue;
            }
            let inv_mass = if node.mass <= 0.0 {
                1.0
            } else {
                1.0 / node.mass
            };
            node.vx = (node.vx + forces[i].x * inv_mass) * self.velocity_decay;
            node.vy = (node.vy + forces[i].y * inv_mass) * self.velocity_decay;
            node.x += node.vx;
            node.y += node.vy;
        }

        // Collision detection — prevent node overlap
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let dx = nodes[j].x - nodes[i].x;
                let dy = nodes[j].y - nodes[i].y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let r_i = node_radius(nodes[i].degree);
                let r_j = node_radius(nodes[j].degree);
                let min_dist = r_i + r_j + 4.0; // 4px gap
                if dist < min_dist {
                    let overlap = (min_dist - dist) / 2.0;
                    let fx = (dx / dist) * overlap;
                    let fy = (dy / dist) * overlap;
                    if !nodes[i].pinned {
                        nodes[i].x -= fx;
                        nodes[i].y -= fy;
                    }
                    if !nodes[j].pinned {
                        nodes[j].x += fx;
                        nodes[j].y += fy;
                    }
                }
            }
        }

        self.alpha += (self.alpha_min - self.alpha) * self.alpha_decay;
    }
}

fn node_index_map(nodes: &[GraphNode]) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        map.insert(node.id.clone(), i);
    }
    map
}

/// Calculate node radius based on degree (connection count).
/// Isolated nodes stay quiet; hubs earn noticeably more weight.
pub fn node_radius(degree: usize) -> f32 {
    7.0 + (degree.min(24) as f32).sqrt() * 1.9
}

/// Duration for hover transitions (150ms)
const HOVER_DURATION_MS: f32 = 150.0;

#[derive(Debug, Default)]
pub struct GraphState {
    dragging: bool,
    last_cursor: Option<Point>,
    last_click: Option<Instant>,
    // Hover animation tracking
    hovered_node: Option<usize>,
    hover_start: Option<Instant>,
    prev_hovered_node: Option<usize>,
    hover_exit_start: Option<Instant>,
}

impl GraphState {
    /// Get hover intensity for a node (0.0 = not hovered, 1.0 = fully hovered)
    fn hover_intensity(&self, node_idx: usize) -> f32 {
        let mut intensity = 0.0;

        // Current hover: ease in
        if self.hovered_node == Some(node_idx) {
            if let Some(start) = self.hover_start {
                let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                let t = (elapsed / HOVER_DURATION_MS).min(1.0);
                intensity = crate::gui::animations::ease_out_quad(t);
            }
        }

        // Previous hover: ease out
        if self.prev_hovered_node == Some(node_idx) && self.hovered_node != Some(node_idx) {
            if let Some(start) = self.hover_exit_start {
                let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                let t = (elapsed / HOVER_DURATION_MS).min(1.0);
                intensity = 1.0 - crate::gui::animations::ease_in_quad(t);
            }
        }

        intensity
    }
}

#[derive(Debug, Clone)]
pub struct GraphCanvas {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    zoom: f32,
    pan: (f32, f32),
}

impl GraphCanvas {
    pub fn new(nodes: Vec<GraphNode>, edges: Vec<GraphEdge>, zoom: f32, pan: (f32, f32)) -> Self {
        Self {
            nodes,
            edges,
            zoom,
            pan,
        }
    }

    fn hovered_node(&self, cursor: mouse::Cursor, bounds: Rectangle) -> Option<usize> {
        let pos = cursor.position_in(bounds)?;
        let world = self.screen_to_world(pos, bounds);
        self.nodes.iter().position(|n| {
            let r = node_radius(n.degree) + 4.0; // generous hit area
            let dx = world.x - n.x;
            let dy = world.y - n.y;
            (dx * dx + dy * dy) <= r * r
        })
    }

    fn screen_to_world(&self, point: Point, bounds: Rectangle) -> Point {
        let offset = self.viewport_offset(bounds);
        Point::new(
            (point.x - offset.x) / self.zoom,
            (point.y - offset.y) / self.zoom,
        )
    }

    fn graph_bounds(&self) -> Option<Rectangle> {
        let first = self.nodes.first()?;
        let mut min_x = first.x;
        let mut max_x = first.x;
        let mut min_y = first.y;
        let mut max_y = first.y;

        for node in &self.nodes {
            let radius = node_radius(node.degree) + 32.0;
            min_x = min_x.min(node.x - radius);
            max_x = max_x.max(node.x + radius);
            min_y = min_y.min(node.y - radius);
            max_y = max_y.max(node.y + radius);
        }

        Some(Rectangle {
            x: min_x,
            y: min_y,
            width: (max_x - min_x).max(1.0),
            height: (max_y - min_y).max(1.0),
        })
    }

    fn graph_center(&self) -> Point {
        self.graph_bounds()
            .map(|bounds| {
                Point::new(
                    bounds.x + bounds.width / 2.0,
                    bounds.y + bounds.height / 2.0,
                )
            })
            .unwrap_or(Point::ORIGIN)
    }

    fn viewport_offset(&self, bounds: Rectangle) -> Vector {
        let center = self.graph_center();
        Vector::new(
            bounds.width / 2.0 + self.pan.0 - center.x * self.zoom,
            bounds.height / 2.0 + self.pan.1 - center.y * self.zoom,
        )
    }
}

pub fn view(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    zoom: f32,
    pan: (f32, f32),
) -> Element<'static, Message> {
    Canvas::new(GraphCanvas::new(nodes.to_vec(), edges.to_vec(), zoom, pan))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

impl canvas::Program<Message> for GraphCanvas {
    type State = GraphState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(idx) = self.hovered_node(cursor, bounds) {
                    let node = &self.nodes[idx];
                    let now = Instant::now();
                    let is_double = state
                        .last_click
                        .map(|t| now.duration_since(t).as_millis() < 300)
                        .unwrap_or(false);
                    state.last_click = Some(now);
                    let _ = is_double;
                    return Some(
                        canvas::Action::publish(Message::GraphNodeClick(node.path.clone()))
                            .and_capture(),
                    );
                }
                state.dragging = true;
                state.last_cursor = cursor.position_in(bounds);
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = false;
                state.last_cursor = None;
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let position = *position;
                if state.dragging {
                    if let Some(last) = state.last_cursor.replace(position) {
                        let dx = position.x - last.x;
                        let dy = position.y - last.y;
                        return Some(
                            canvas::Action::publish(Message::GraphPan { dx, dy }).and_capture(),
                        );
                    }
                }

                // Track hover state for smooth transitions
                let new_hovered = self.hovered_node(cursor, bounds);
                if new_hovered != state.hovered_node {
                    // Outgoing hover: save as previous for exit animation
                    if let Some(prev) = state.hovered_node {
                        state.prev_hovered_node = Some(prev);
                        state.hover_exit_start = Some(Instant::now());
                    }
                    // Incoming hover
                    state.hovered_node = new_hovered;
                    state.hover_start = if new_hovered.is_some() {
                        Some(Instant::now())
                    } else {
                        None
                    };
                }

                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let amount = match *delta {
                    mouse::ScrollDelta::Lines { y, .. } => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 60.0,
                };
                Some(canvas::Action::publish(Message::GraphZoom(amount)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let light = is_light_theme(theme);

        // Theme-aware colors
        let edge_color = if light {
            colors::latte::SURFACE2
        } else {
            colors::SURFACE2
        };
        let node_base = if light {
            colors::latte::ACCENT
        } else {
            colors::ACCENT
        };
        let node_hover = if light {
            colors::latte::LAVENDER
        } else {
            colors::LAVENDER
        };
        let node_hub = if light {
            colors::latte::TEAL
        } else {
            colors::TEAL
        };
        let text_color = if light {
            colors::latte::TEXT
        } else {
            colors::TEXT
        };
        let offset = self.viewport_offset(bounds);
        let center = self.graph_center();

        frame.translate(offset);
        frame.scale(self.zoom);

        let index = node_index_map(&self.nodes);

        for radius in [120.0, 240.0, 380.0, 560.0] {
            let orbit = Path::circle(center, radius);
            frame.stroke(
                &orbit,
                Stroke::default()
                    .with_color(Color {
                        a: 0.08,
                        ..edge_color
                    })
                    .with_width(1.0),
            );
        }

        let horizontal_axis = Path::line(
            Point::new(center.x - 1200.0, center.y),
            Point::new(center.x + 1200.0, center.y),
        );
        let vertical_axis = Path::line(
            Point::new(center.x, center.y - 1200.0),
            Point::new(center.x, center.y + 1200.0),
        );
        frame.stroke(
            &horizontal_axis,
            Stroke::default()
                .with_color(Color {
                    a: 0.06,
                    ..text_color
                })
                .with_width(1.0),
        );
        frame.stroke(
            &vertical_axis,
            Stroke::default()
                .with_color(Color {
                    a: 0.06,
                    ..text_color
                })
                .with_width(1.0),
        );

        // Edges
        for edge in &self.edges {
            if let (Some(&a), Some(&b)) = (index.get(&edge.from), index.get(&edge.to)) {
                let from = Point::new(self.nodes[a].x, self.nodes[a].y);
                let to = Point::new(self.nodes[b].x, self.nodes[b].y);
                let path = Path::line(from, to);
                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_color(Color {
                            a: 0.28,
                            ..edge_color
                        })
                        .with_width(1.2),
                );
            }
        }

        // Nodes with smooth hover transitions + degree-based sizing
        for (i, node) in self.nodes.iter().enumerate() {
            let hover_t = state.hover_intensity(i);

            // Degree-based radius + hover expansion
            let base_radius = node_radius(node.degree);
            let radius = base_radius + 2.0 * hover_t;
            let degree_t = (node.degree.min(12) as f32) / 12.0;

            // Interpolate base → hub → hover to make structure legible.
            let hub_color = mix_color(node_base, node_hub, degree_t * 0.5);
            let color = mix_color(hub_color, node_hover, hover_t * 0.7);

            // Glow with animated opacity (0.0 → 0.2)
            if hover_t > 0.01 {
                let glow_radius = radius + 4.0 * hover_t;
                let glow = Path::circle(Point::new(node.x, node.y), glow_radius);
                frame.fill(
                    &glow,
                    Color {
                        a: 0.18 * hover_t,
                        ..color
                    },
                );
            }

            let shadow = Path::circle(Point::new(node.x, node.y), radius + 3.0);
            frame.fill(
                &shadow,
                Color {
                    a: 0.10 + degree_t * 0.05,
                    ..Color::BLACK
                },
            );

            let circle = Path::circle(Point::new(node.x, node.y), radius);
            frame.fill(&circle, color);
            frame.stroke(
                &circle,
                Stroke::default()
                    .with_color(Color {
                        a: 0.45,
                        ..text_color
                    })
                    .with_width(1.0),
            );

            let show_label = hover_t > 0.15 || node.degree > 1;
            let label_alpha = if show_label { 0.95 } else { 0.58 };
            let label_width = (node.label.chars().count() as f32 * 6.6).clamp(44.0, 220.0);
            let label_origin = Point::new(node.x + radius + 8.0, node.y - 12.0);

            if show_label {
                let label_bg = Path::rectangle(label_origin, Size::new(label_width, 22.0));
                frame.fill(
                    &label_bg,
                    Color {
                        a: 0.20 + hover_t * 0.08,
                        ..if light {
                            colors::latte::MANTLE
                        } else {
                            colors::MANTLE
                        }
                    },
                );
            }

            frame.fill_text(Text {
                content: node.label.clone(),
                position: Point::new(node.x + radius + 12.0, node.y + 4.0),
                color: Color {
                    a: label_alpha,
                    ..text_color
                },
                size: if hover_t > 0.2 { 13.0 } else { 12.0 }.into(),
                ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }
}

fn mix_color(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    }
}
