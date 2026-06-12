use ratatui::style::{Color, Modifier, Style as TuiStyle};

use crate::markdown::style::Style;

pub fn style_for(style: Style) -> TuiStyle {
    match style {
        Style::Text => TuiStyle::default(),
        Style::Heading1 | Style::Heading2 | Style::Heading3 => {
            TuiStyle::default().add_modifier(Modifier::BOLD)
        }
        Style::Bold => TuiStyle::default().add_modifier(Modifier::BOLD),
        Style::Italic => TuiStyle::default().add_modifier(Modifier::ITALIC),
        Style::CodeSpan | Style::CodeBlock => TuiStyle::default().fg(Color::Cyan),
        Style::Link => TuiStyle::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::UNDERLINED),
        Style::Tag => TuiStyle::default().fg(Color::Magenta),
        Style::Quote => TuiStyle::default().add_modifier(Modifier::DIM),
        Style::QuoteBar | Style::Frame => TuiStyle::default().add_modifier(Modifier::DIM),
        Style::Frontmatter => TuiStyle::default().add_modifier(Modifier::DIM),
        Style::ImagePlaceholder => TuiStyle::default().add_modifier(Modifier::DIM),
    }
}
