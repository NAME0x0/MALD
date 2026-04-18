//! Minimal Mermaid parser + canvas renderer (flowchart/sequence/class MVP).

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::{alignment, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector};
use regex::Regex;
use std::collections::HashMap;

use crate::gui::message::Message;
use crate::gui::theme::{self, colors, spacing, type_scale};

#[derive(Debug, Clone)]
pub enum MermaidDiagram {
    Flowchart(FlowchartData),
    Sequence(SequenceData),
    Class(ClassData),
    State(FlowchartData),
    Unsupported(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    TD,
    TB,
    LR,
    RL,
}

#[derive(Debug, Clone)]
pub struct FlowchartData {
    pub direction: Direction,
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
    pub position: Point,
    pub size: Size,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeShape {
    Rectangle,
    Rounded,
    Diamond,
    Circle,
    Stadium,
    Parallelogram,
    Hexagon,
}

#[derive(Debug, Clone)]
pub struct FlowEdge {
    pub from: usize,
    pub to: usize,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SequenceData {
    pub participants: Vec<String>,
    pub messages: Vec<SequenceMessage>,
}

#[derive(Debug, Clone)]
pub struct SequenceMessage {
    pub from: usize,
    pub to: usize,
    pub text: String,
    pub dashed: bool,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub classes: Vec<ClassNode>,
    pub relations: Vec<ClassRelation>,
}

#[derive(Debug, Clone)]
pub struct ClassNode {
    pub name: String,
    pub members: Vec<String>,
    pub position: Point,
    pub size: Size,
}

#[derive(Debug, Clone)]
pub struct ClassRelation {
    pub from: usize,
    pub to: usize,
    pub label: Option<String>,
}

pub fn view(source: &str, is_dark: bool) -> Element<'static, Message> {
    let subtext0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    match parse_mermaid(source) {
        Ok(diagram) => Canvas::new(MermaidCanvas { diagram })
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        Err(err) => iced::widget::container(
            iced::widget::text(format!("Mermaid parse error: {err}"))
                .size(type_scale::UI)
                .color(subtext0),
        )
        .padding(spacing::MD as u16)
        .into(),
    }
}

pub fn parse_mermaid(source: &str) -> Result<MermaidDiagram, String> {
    let lines: Vec<&str> = source
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    let first = lines.first().ok_or("Empty diagram")?;

    if first.starts_with("graph") || first.starts_with("flowchart") {
        parse_flowchart(&lines).map(MermaidDiagram::Flowchart)
    } else if first.starts_with("sequenceDiagram") {
        parse_sequence(&lines).map(MermaidDiagram::Sequence)
    } else if first.starts_with("classDiagram") {
        parse_class(&lines).map(MermaidDiagram::Class)
    } else if first.starts_with("stateDiagram") {
        parse_flowchart(&lines).map(MermaidDiagram::State)
    } else {
        Ok(MermaidDiagram::Unsupported(first.to_string()))
    }
}

fn parse_flowchart(lines: &[&str]) -> Result<FlowchartData, String> {
    let mut direction = Direction::TD;
    if let Some(first) = lines.first() {
        let tokens: Vec<&str> = first.split_whitespace().collect();
        if tokens.len() >= 2 {
            direction = match tokens[1] {
                "TD" | "TB" => Direction::TD,
                "LR" => Direction::LR,
                "RL" => Direction::RL,
                _ => Direction::TD,
            };
        }
    }

    let mut nodes: Vec<FlowNode> = Vec::new();
    let mut edges: Vec<FlowEdge> = Vec::new();
    let mut node_index: HashMap<String, usize> = HashMap::new();

    for line in lines.iter().skip(1) {
        if line.starts_with("%%") {
            continue;
        }
        if let Some((left, right, label)) = split_edge(line) {
            let (from_id, from_label, from_shape) = parse_node_token(&left);
            let (to_id, to_label, to_shape) = parse_node_token(&right);
            let from_idx =
                get_or_insert_node(&mut nodes, &mut node_index, from_id, from_label, from_shape);
            let to_idx = get_or_insert_node(&mut nodes, &mut node_index, to_id, to_label, to_shape);
            edges.push(FlowEdge {
                from: from_idx,
                to: to_idx,
                label,
            });
        } else {
            let (id, label, shape) = parse_node_token(line);
            let _ = get_or_insert_node(&mut nodes, &mut node_index, id, label, shape);
        }
    }

    layout_flowchart(&mut nodes, direction);

    Ok(FlowchartData {
        direction,
        nodes,
        edges,
    })
}

fn parse_sequence(lines: &[&str]) -> Result<SequenceData, String> {
    let mut participants: Vec<String> = Vec::new();
    let mut messages: Vec<SequenceMessage> = Vec::new();

    let re = Regex::new(r"^(\w+)\s*(-+>>|->>)\s*(\w+)\s*:\s*(.+)$").map_err(|e| e.to_string())?;

    for line in lines.iter().skip(1) {
        if let Some(cap) = re.captures(line) {
            let from = cap[1].to_string();
            let arrow = cap[2].to_string();
            let to = cap[3].to_string();
            let text = cap[4].trim().to_string();
            let from_idx = index_or_push(&mut participants, &from);
            let to_idx = index_or_push(&mut participants, &to);
            messages.push(SequenceMessage {
                from: from_idx,
                to: to_idx,
                text,
                dashed: arrow.starts_with("--"),
            });
        }
    }

    Ok(SequenceData {
        participants,
        messages,
    })
}

fn parse_class(lines: &[&str]) -> Result<ClassData, String> {
    let mut classes: Vec<ClassNode> = Vec::new();
    let mut relations: Vec<ClassRelation> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();

    let rel_re = Regex::new(r"^(\w+)\s*<\|--\s*(\w+)$").map_err(|e| e.to_string())?;
    let member_re = Regex::new(r"^(\w+)\s*:\s*(.+)$").map_err(|e| e.to_string())?;

    for line in lines.iter().skip(1) {
        if let Some(cap) = rel_re.captures(line) {
            let a = cap[1].to_string();
            let b = cap[2].to_string();
            let a_idx = class_index(&mut classes, &mut index, &a);
            let b_idx = class_index(&mut classes, &mut index, &b);
            relations.push(ClassRelation {
                from: a_idx,
                to: b_idx,
                label: None,
            });
        } else if let Some(cap) = member_re.captures(line) {
            let name = cap[1].to_string();
            let member = cap[2].trim().to_string();
            let idx = class_index(&mut classes, &mut index, &name);
            classes[idx].members.push(member);
        } else if !line.is_empty() {
            let name = line.split_whitespace().next().unwrap_or(line).to_string();
            let _ = class_index(&mut classes, &mut index, &name);
        }
    }

    layout_classes(&mut classes);
    Ok(ClassData { classes, relations })
}

fn split_edge(line: &str) -> Option<(String, String, Option<String>)> {
    let candidates = ["-->", "---", "->"];
    for arrow in candidates {
        if let Some(idx) = line.find(arrow) {
            let left = line[..idx].trim().to_string();
            let mut right = line[idx + arrow.len()..].trim().to_string();
            let mut label = None;
            if right.starts_with('|') {
                if let Some(end) = right[1..].find('|') {
                    label = Some(right[1..1 + end].trim().to_string());
                    right = right[2 + end..].trim().to_string();
                }
            }
            return Some((left, right, label));
        }
    }
    None
}

fn parse_node_token(token: &str) -> (String, String, NodeShape) {
    let trimmed = token.trim();
    let mut id = String::new();
    for ch in trimmed.chars() {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            id.push(ch);
        } else {
            break;
        }
    }
    let rest = trimmed[id.len()..].trim();
    if rest.starts_with("(((") || rest.starts_with("((") {
        return (
            id.clone(),
            extract_between(rest, "((", "))").unwrap_or(id),
            NodeShape::Circle,
        );
    }
    if rest.starts_with("[[") || rest.starts_with("[") {
        return (
            id.clone(),
            extract_between(rest, "[", "]").unwrap_or(id),
            NodeShape::Rectangle,
        );
    }
    if rest.starts_with("([") {
        return (
            id.clone(),
            extract_between(rest, "([", "])").unwrap_or(id),
            NodeShape::Stadium,
        );
    }
    if rest.starts_with("{") {
        return (
            id.clone(),
            extract_between(rest, "{", "}").unwrap_or(id),
            NodeShape::Diamond,
        );
    }
    if rest.starts_with("(") {
        return (
            id.clone(),
            extract_between(rest, "(", ")").unwrap_or(id),
            NodeShape::Rounded,
        );
    }
    (id.clone(), id, NodeShape::Rectangle)
}

fn extract_between(text: &str, start: &str, end: &str) -> Option<String> {
    let s = text.find(start)?;
    let e = text.rfind(end)?;
    if e <= s + start.len() {
        return None;
    }
    Some(text[s + start.len()..e].trim().to_string())
}

fn get_or_insert_node(
    nodes: &mut Vec<FlowNode>,
    index: &mut HashMap<String, usize>,
    id: String,
    label: String,
    shape: NodeShape,
) -> usize {
    if let Some(&idx) = index.get(&id) {
        if nodes[idx].label.is_empty() {
            nodes[idx].label = label;
        }
        return idx;
    }
    let size = Size::new((label.len().max(4) as f32) * 7.0 + 20.0, 32.0);
    let node = FlowNode {
        id: id.clone(),
        label,
        shape,
        position: Point::new(0.0, 0.0),
        size,
    };
    let idx = nodes.len();
    nodes.push(node);
    index.insert(id, idx);
    idx
}

fn layout_flowchart(nodes: &mut [FlowNode], direction: Direction) {
    if nodes.is_empty() {
        return;
    }
    let cols = (nodes.len() as f32).sqrt().ceil() as usize;
    let col_count = cols.max(1);
    let spacing_x = 180.0;
    let spacing_y = 110.0;

    for (i, node) in nodes.iter_mut().enumerate() {
        let row = i / col_count;
        let col = i % col_count;
        let (x, y) = match direction {
            Direction::LR | Direction::RL => (row as f32 * spacing_x, col as f32 * spacing_y),
            _ => (col as f32 * spacing_x, row as f32 * spacing_y),
        };
        node.position = Point::new(x + 60.0, y + 40.0);
    }
}

fn layout_classes(classes: &mut [ClassNode]) {
    let cols = (classes.len() as f32).sqrt().ceil() as usize;
    let spacing_x = 200.0;
    let spacing_y = 140.0;
    for (i, class) in classes.iter_mut().enumerate() {
        let row = i / cols.max(1);
        let col = i % cols.max(1);
        class.position = Point::new(col as f32 * spacing_x + 40.0, row as f32 * spacing_y + 40.0);
        let height = 36.0 + (class.members.len() as f32 * 14.0);
        class.size = Size::new(140.0, height.max(40.0));
    }
}

fn index_or_push(list: &mut Vec<String>, name: &str) -> usize {
    if let Some(idx) = list.iter().position(|s| s == name) {
        idx
    } else {
        list.push(name.to_string());
        list.len() - 1
    }
}

fn class_index(
    classes: &mut Vec<ClassNode>,
    index: &mut HashMap<String, usize>,
    name: &str,
) -> usize {
    if let Some(&idx) = index.get(name) {
        return idx;
    }
    let idx = classes.len();
    classes.push(ClassNode {
        name: name.to_string(),
        members: Vec::new(),
        position: Point::new(0.0, 0.0),
        size: Size::new(140.0, 40.0),
    });
    index.insert(name.to_string(), idx);
    idx
}

/// Themed color set for mermaid canvas rendering
struct MermaidColors {
    text: iced::Color,
    subtext0: iced::Color,
    surface0: iced::Color,
    surface2: iced::Color,
    blue: iced::Color,
}

impl MermaidColors {
    fn from_theme(theme: &Theme) -> Self {
        let light = theme::is_light_theme(theme);
        Self {
            text: if light {
                colors::latte::TEXT
            } else {
                colors::TEXT
            },
            subtext0: if light {
                colors::latte::SUBTEXT0
            } else {
                colors::SUBTEXT0
            },
            surface0: if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            },
            surface2: if light {
                colors::latte::SURFACE2
            } else {
                colors::SURFACE2
            },
            blue: if light {
                colors::latte::BLUE
            } else {
                colors::BLUE
            },
        }
    }
}

#[derive(Debug, Clone)]
struct MermaidCanvas {
    diagram: MermaidDiagram,
}

impl canvas::Program<Message> for MermaidCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let colors = MermaidColors::from_theme(theme);
        let mut frame = Frame::new(renderer, bounds.size());
        frame.translate(Vector::new(16.0, 16.0));

        match &self.diagram {
            MermaidDiagram::Flowchart(data) | MermaidDiagram::State(data) => {
                draw_flowchart(&mut frame, data, &colors);
            }
            MermaidDiagram::Sequence(data) => {
                draw_sequence(&mut frame, data, &colors);
            }
            MermaidDiagram::Class(data) => {
                draw_class(&mut frame, data, &colors);
            }
            MermaidDiagram::Unsupported(name) => {
                frame.fill_text(Text {
                    content: format!("Unsupported Mermaid diagram: {name}"),
                    position: Point::new(8.0, 8.0),
                    color: colors.subtext0,
                    size: 12.0.into(),
                    ..Default::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }
}

fn draw_flowchart(frame: &mut Frame, data: &FlowchartData, c: &MermaidColors) {
    for edge in &data.edges {
        let from = &data.nodes[edge.from];
        let to = &data.nodes[edge.to];
        let start = center_of(from);
        let end = center_of(to);
        let path = Path::line(start, end);
        frame.stroke(
            &path,
            Stroke::default().with_color(c.subtext0).with_width(1.5),
        );
        draw_arrow(frame, end, angle(start, end), c.subtext0);
        if let Some(label) = &edge.label {
            let mid = Point::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
            frame.fill_text(Text {
                content: label.clone(),
                position: mid,
                color: c.subtext0,
                size: 10.0.into(),
                ..Default::default()
            });
        }
    }

    for node in &data.nodes {
        let path = match node.shape {
            NodeShape::Rectangle => Path::rectangle(node.position, node.size),
            NodeShape::Rounded => rounded_rect(node.position, node.size, 8.0),
            NodeShape::Diamond => diamond_path(node.position, node.size),
            NodeShape::Circle => {
                let radius = (node.size.width.max(node.size.height)) / 2.2;
                Path::circle(center_of(node), radius)
            }
            NodeShape::Stadium => rounded_rect(node.position, node.size, node.size.height / 2.0),
            NodeShape::Parallelogram => parallelogram_path(node.position, node.size, 10.0),
            NodeShape::Hexagon => hexagon_path(node.position, node.size),
        };
        frame.fill(&path, c.surface0);
        frame.stroke(&path, Stroke::default().with_color(c.blue).with_width(2.0));
        frame.fill_text(Text {
            content: node.label.clone(),
            position: center_of(node),
            color: c.text,
            size: 12.0.into(),
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..Default::default()
        });
    }
}

fn draw_sequence(frame: &mut Frame, data: &SequenceData, c: &MermaidColors) {
    let spacing_x = 140.0;
    let start_y = 20.0;
    let message_gap = 26.0;

    for (i, name) in data.participants.iter().enumerate() {
        let x = i as f32 * spacing_x + 20.0;
        frame.fill_text(Text {
            content: name.clone(),
            position: Point::new(x, start_y),
            color: c.text,
            size: 12.0.into(),
            ..Default::default()
        });
        let path = Path::line(
            Point::new(x, start_y + 12.0),
            Point::new(x, start_y + 200.0),
        );
        frame.stroke(
            &path,
            Stroke::default().with_color(c.surface2).with_width(1.0),
        );
    }

    for (i, msg) in data.messages.iter().enumerate() {
        let y = start_y + 24.0 + (i as f32 * message_gap);
        let from_x = msg.from as f32 * spacing_x + 20.0;
        let to_x = msg.to as f32 * spacing_x + 20.0;
        let path = Path::line(Point::new(from_x, y), Point::new(to_x, y));
        frame.stroke(
            &path,
            Stroke::default()
                .with_color(c.subtext0)
                .with_width(if msg.dashed { 1.0 } else { 1.5 }),
        );
        draw_arrow(
            frame,
            Point::new(to_x, y),
            angle(Point::new(from_x, y), Point::new(to_x, y)),
            c.subtext0,
        );
        frame.fill_text(Text {
            content: msg.text.clone(),
            position: Point::new((from_x + to_x) / 2.0, y - 8.0),
            color: c.subtext0,
            size: 10.0.into(),
            align_x: alignment::Horizontal::Center.into(),
            ..Default::default()
        });
    }
}

fn draw_class(frame: &mut Frame, data: &ClassData, c: &MermaidColors) {
    for rel in &data.relations {
        let from = &data.classes[rel.from];
        let to = &data.classes[rel.to];
        let start = center_of_class(from);
        let end = center_of_class(to);
        let path = Path::line(start, end);
        frame.stroke(
            &path,
            Stroke::default().with_color(c.subtext0).with_width(1.2),
        );
        draw_arrow(frame, end, angle(start, end), c.subtext0);
    }

    for class in &data.classes {
        let rect = Path::rectangle(class.position, class.size);
        frame.fill(&rect, c.surface0);
        frame.stroke(&rect, Stroke::default().with_color(c.blue).with_width(2.0));
        frame.fill_text(Text {
            content: class.name.clone(),
            position: Point::new(class.position.x + 6.0, class.position.y + 6.0),
            color: c.text,
            size: 12.0.into(),
            ..Default::default()
        });
        for (i, member) in class.members.iter().enumerate() {
            frame.fill_text(Text {
                content: member.clone(),
                position: Point::new(
                    class.position.x + 6.0,
                    class.position.y + 24.0 + i as f32 * 12.0,
                ),
                color: c.subtext0,
                size: 10.0.into(),
                ..Default::default()
            });
        }
    }
}

fn rounded_rect(pos: Point, size: Size, radius: f32) -> Path {
    Path::rounded_rectangle(pos, size, radius.into())
}

fn diamond_path(pos: Point, size: Size) -> Path {
    let center = Point::new(pos.x + size.width / 2.0, pos.y + size.height / 2.0);
    let half_w = size.width / 2.0;
    let half_h = size.height / 2.0;
    Path::new(|b| {
        b.move_to(Point::new(center.x, center.y - half_h));
        b.line_to(Point::new(center.x + half_w, center.y));
        b.line_to(Point::new(center.x, center.y + half_h));
        b.line_to(Point::new(center.x - half_w, center.y));
        b.close();
    })
}

fn parallelogram_path(pos: Point, size: Size, skew: f32) -> Path {
    Path::new(|b| {
        b.move_to(Point::new(pos.x + skew, pos.y));
        b.line_to(Point::new(pos.x + size.width, pos.y));
        b.line_to(Point::new(pos.x + size.width - skew, pos.y + size.height));
        b.line_to(Point::new(pos.x, pos.y + size.height));
        b.close();
    })
}

fn hexagon_path(pos: Point, size: Size) -> Path {
    let w = size.width;
    let h = size.height;
    Path::new(|b| {
        b.move_to(Point::new(pos.x + w * 0.25, pos.y));
        b.line_to(Point::new(pos.x + w * 0.75, pos.y));
        b.line_to(Point::new(pos.x + w, pos.y + h * 0.5));
        b.line_to(Point::new(pos.x + w * 0.75, pos.y + h));
        b.line_to(Point::new(pos.x + w * 0.25, pos.y + h));
        b.line_to(Point::new(pos.x, pos.y + h * 0.5));
        b.close();
    })
}

fn center_of(node: &FlowNode) -> Point {
    Point::new(
        node.position.x + node.size.width / 2.0,
        node.position.y + node.size.height / 2.0,
    )
}

fn center_of_class(node: &ClassNode) -> Point {
    Point::new(
        node.position.x + node.size.width / 2.0,
        node.position.y + node.size.height / 2.0,
    )
}

fn draw_arrow(frame: &mut Frame, tip: Point, angle: f32, color: iced::Color) {
    let size = 6.0;
    let left = Point::new(
        tip.x - size * angle.cos() + size * angle.sin(),
        tip.y - size * angle.sin() - size * angle.cos(),
    );
    let right = Point::new(
        tip.x - size * angle.cos() - size * angle.sin(),
        tip.y - size * angle.sin() + size * angle.cos(),
    );
    let path = Path::new(|b| {
        b.move_to(tip);
        b.line_to(left);
        b.line_to(right);
        b.close();
    });
    frame.fill(&path, color);
}

fn angle(start: Point, end: Point) -> f32 {
    (end.y - start.y).atan2(end.x - start.x)
}
