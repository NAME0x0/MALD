//! Simple syntax-highlighted code viewer.

use iced::widget::container;
use iced::widget::text::{Rich, Span};
use iced::{Element, Length};

use crate::gui::message::Message;
use crate::gui::syntax::{HighlightSpan, SyntaxHighlighter};
use crate::gui::theme::{self, colors, spacing, type_scale};

pub fn view(
    code: &str,
    lang: &str,
    highlighter: &SyntaxHighlighter,
    is_dark: bool,
) -> Element<'static, Message> {
    let spans = highlighter.highlight(code, lang);
    let rich = build_rich_text(code, &spans, is_dark);

    container(rich)
        .style(theme::code_block_style)
        .padding(spacing::MD as u16)
        .width(Length::Fill)
        .into()
}

fn build_rich_text(
    code: &str,
    spans: &[HighlightSpan],
    is_dark: bool,
) -> Rich<'static, (), Message, iced::Theme> {
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };

    if spans.is_empty() {
        return Rich::with_spans([Span::new(code.to_string())
            .font(iced::Font::MONOSPACE)
            .color(text_color)])
        .size(type_scale::UI);
    }

    let mut rich_spans: Vec<Span<'static>> = Vec::new();
    let mut cursor = 0usize;

    for span in spans {
        if span.start > cursor {
            rich_spans.push(
                Span::new(code[cursor..span.start].to_string())
                    .font(iced::Font::MONOSPACE)
                    .color(text_color),
            );
        }
        let s = Span::new(code[span.start..span.end].to_string())
            .font(iced::Font::MONOSPACE)
            .color(span.color);
        let _ = span.style;
        rich_spans.push(s);
        cursor = span.end;
    }

    if cursor < code.len() {
        rich_spans.push(
            Span::new(code[cursor..].to_string())
                .font(iced::Font::MONOSPACE)
                .color(text_color),
        );
    }

    Rich::with_spans(rich_spans).size(type_scale::UI)
}
