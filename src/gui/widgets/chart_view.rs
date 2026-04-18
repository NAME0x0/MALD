//! Inline chart widget for code-cell outputs.

use std::sync::OnceLock;

use iced::{Element, Length};
use plotters::prelude::*;
use plotters_iced2::{Chart, ChartWidget};
use regex::Regex;

use crate::gui::message::Message;

#[derive(Debug, Clone)]
struct MiniChart {
    data: Vec<(f32, f32)>,
}

impl<Message> Chart<Message> for MiniChart {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        if self.data.is_empty() {
            return;
        }

        let x_max = (self.data.len().saturating_sub(1) as f32).max(1.0);
        let (mut y_min, mut y_max) = (f32::MAX, f32::MIN);
        for (_, y) in &self.data {
            y_min = y_min.min(*y);
            y_max = y_max.max(*y);
        }
        if (y_max - y_min).abs() < f32::EPSILON {
            y_max += 1.0;
            y_min -= 1.0;
        }

        if let Ok(mut chart) = builder
            .margin(8)
            .set_all_label_area_size(0)
            .build_cartesian_2d(0f32..x_max, y_min..y_max)
        {
            chart
                .configure_mesh()
                .disable_mesh()
                .disable_axes()
                .draw()
                .ok();

            let stroke = RGBColor(137, 180, 250);
            chart
                .draw_series(LineSeries::new(
                    self.data.iter().cloned(),
                    stroke.stroke_width(2),
                ))
                .ok();
        }
    }
}

pub fn view_from_code(code: &str) -> Option<Element<'static, Message>> {
    let data = parse_numbers(code);
    if data.len() < 2 {
        return None;
    }

    let chart = MiniChart {
        data: data
            .into_iter()
            .enumerate()
            .map(|(i, y)| (i as f32, y))
            .collect(),
    };

    Some(
        ChartWidget::new(chart)
            .width(Length::Fill)
            .height(Length::Fixed(140.0))
            .into(),
    )
}

fn parse_numbers(code: &str) -> Vec<f32> {
    static NUMBER_RE: OnceLock<Regex> = OnceLock::new();
    NUMBER_RE
        .get_or_init(|| Regex::new(r"-?\d+(\.\d+)?").expect("number regex must compile"))
        .find_iter(code)
        .filter_map(|m| m.as_str().parse::<f32>().ok())
        .collect()
}
