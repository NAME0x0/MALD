//! Markdown preview rendering for the editor surface.

use std::cell::Cell;
use std::sync::OnceLock;

use iced::widget::{container, markdown, scrollable};
use iced::{Element, Length, Renderer, Theme};
use regex::Regex;

use crate::gui::canvas::mermaid;
use crate::gui::message::Message;
use crate::gui::syntax::SyntaxHighlighter;
use crate::gui::theme::{self, spacing, type_scale};
use crate::gui::widgets::code_cell::{self, CodeCell};
use crate::gui::widgets::syntax_editor;

pub fn parse_markdown(content: &str) -> markdown::Content {
    markdown::Content::parse(&rewrite_wikilinks(content))
}

pub fn render_markdown<'a>(
    content: &'a markdown::Content,
    code_cells: &'a [CodeCell],
    highlighter: &'a SyntaxHighlighter,
    iced_theme: Theme,
    is_dark: bool,
) -> Element<'a, Message> {
    let mut settings = markdown::Settings::from(iced_theme);
    settings.text_size = type_scale::BODY.into();
    settings.code_size = type_scale::UI.into();
    settings.spacing = spacing::MD.into();

    let viewer = MaldMarkdownViewer {
        code_cells,
        highlighter,
        is_dark,
        code_index: Cell::new(0),
    };

    scrollable(
        container(markdown::view_with(content.items(), settings, &viewer))
            .padding(spacing::LG as u16)
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .style(theme::scrollable_style)
    .into()
}

struct MaldMarkdownViewer<'a> {
    code_cells: &'a [CodeCell],
    highlighter: &'a SyntaxHighlighter,
    is_dark: bool,
    code_index: Cell<usize>,
}

impl<'a> markdown::Viewer<'a, Message, Theme, Renderer> for MaldMarkdownViewer<'a> {
    fn on_link_click(url: markdown::Uri) -> Message {
        Message::MarkdownLinkClick(url)
    }

    fn code_block(
        &self,
        _settings: markdown::Settings,
        language: Option<&'a str>,
        code: &'a str,
        _lines: &'a [markdown::Text],
    ) -> Element<'a, Message> {
        let index = self.code_index.get();
        self.code_index.set(index + 1);

        let language = language.unwrap_or("");

        if language.trim().eq_ignore_ascii_case("mermaid") {
            return container(mermaid::view(code, self.is_dark))
                .height(Length::Fixed(260.0))
                .style(theme::code_block_style)
                .padding(spacing::SM as u16)
                .into();
        }

        if let Some(cell) = self.code_cells.iter().find(|cell| cell.id == index) {
            return code_cell::view(cell, self.is_dark);
        }

        syntax_editor::view(code, language, self.highlighter, self.is_dark)
    }
}

fn rewrite_wikilinks(content: &str) -> String {
    static WIKILINK_RE: OnceLock<Regex> = OnceLock::new();

    let regex =
        WIKILINK_RE.get_or_init(|| Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap());

    regex
        .replace_all(content, |captures: &regex::Captures| {
            let target = captures.get(1).map(|match_| match_.as_str()).unwrap_or("");
            let label = captures
                .get(2)
                .map(|match_| match_.as_str())
                .unwrap_or(target)
                .trim();

            format!(
                "[{label}](mald-note://{})",
                encode_note_target(target.trim())
            )
        })
        .into_owned()
}

fn encode_note_target(target: &str) -> String {
    target
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('[', "%5B")
        .replace(']', "%5D")
}
