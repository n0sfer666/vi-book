use std::collections::HashMap;

use ratatui::style::{Color, Modifier, Style as TuiStyle};

use crate::markdown::style::Style;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Face {
    pub color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
}

impl Face {
    fn color(color: Color) -> Self {
        Self {
            color: Some(color),
            ..Self::default()
        }
    }

    fn bold() -> Self {
        Self {
            bold: true,
            ..Self::default()
        }
    }

    fn dim() -> Self {
        Self {
            dim: true,
            ..Self::default()
        }
    }

    pub fn tui(self) -> TuiStyle {
        let mut style = TuiStyle::default();
        if let Some(color) = self.color {
            style = style.fg(color);
        }
        if self.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if self.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if self.underline {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        if self.dim {
            style = style.add_modifier(Modifier::DIM);
        }
        style
    }
}

pub struct Theme {
    faces: HashMap<Style, Face>,
}

impl Default for Theme {
    fn default() -> Self {
        let mut faces = HashMap::new();
        for style in Style::ALL {
            faces.insert(style, default_face(style));
        }
        Self { faces }
    }
}

impl Theme {
    pub fn face(&self, style: Style) -> Face {
        self.faces.get(&style).copied().unwrap_or_default()
    }

    pub fn set(&mut self, style: Style, face: Face) {
        self.faces.insert(style, face);
    }

    pub fn tui(&self, style: Style) -> TuiStyle {
        self.face(style).tui()
    }
}

fn default_face(style: Style) -> Face {
    match style {
        Style::Text => Face::default(),
        Style::Heading1 | Style::Heading2 | Style::Heading3 | Style::Bold => Face::bold(),
        Style::Italic => Face {
            italic: true,
            ..Face::default()
        },
        Style::CodeSpan | Style::CodeBlock => Face::color(Color::Cyan),
        Style::Link => Face {
            color: Some(Color::Blue),
            underline: true,
            ..Face::default()
        },
        Style::Tag => Face::color(Color::Magenta),
        Style::Quote
        | Style::QuoteBar
        | Style::Frame
        | Style::Frontmatter
        | Style::ImagePlaceholder => Face::dim(),
    }
}
