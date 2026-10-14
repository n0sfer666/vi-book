use std::collections::HashMap;

use ratatui::style::Style as TuiStyle;

use v_ide_markdown::style::{CodeToken, Style};
use v_ide_view::{Ansi, Color, Mods, Style as CellStyle};

use crate::tui_style;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Face {
    pub color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
}

impl Face {
    fn color(ansi: Ansi) -> Self {
        Self {
            color: Some(Color::Ansi(ansi)),
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

    pub fn style(self) -> CellStyle {
        CellStyle {
            fg: self.color.unwrap_or_default(),
            mods: Mods {
                bold: self.bold,
                italic: self.italic,
                underline: self.underline,
                dim: self.dim,
                ..Mods::default()
            },
            ..CellStyle::default()
        }
    }

    pub fn tui(self) -> TuiStyle {
        tui_style::from_cell(self.style())
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
        if let Some(face) = self.faces.get(&style) {
            return *face;
        }
        match style {
            Style::Code(token) => code_face(token),
            _ => Face::default(),
        }
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
        Style::CodeSpan | Style::CodeBlock => Face::color(Ansi::Cyan),
        Style::Link => Face {
            color: Some(Color::Ansi(Ansi::Blue)),
            underline: true,
            ..Face::default()
        },
        Style::Tag => Face::color(Ansi::Magenta),
        Style::Quote
        | Style::QuoteBar
        | Style::Frame
        | Style::Frontmatter
        | Style::ImagePlaceholder => Face::dim(),
        Style::Code(token) => code_face(token),
    }
}

fn code_face(token: CodeToken) -> Face {
    match token {
        CodeToken::Keyword => Face::color(Ansi::Magenta),
        CodeToken::String => Face::color(Ansi::Green),
        CodeToken::Number | CodeToken::Constant | CodeToken::Attribute => Face::color(Ansi::Yellow),
        CodeToken::Comment => Face {
            color: Some(Color::Ansi(Ansi::DarkGray)),
            dim: true,
            ..Face::default()
        },
        CodeToken::Function => Face::color(Ansi::Blue),
        CodeToken::Type | CodeToken::Property => Face::color(Ansi::Cyan),
        CodeToken::Variable => Face::color(Ansi::White),
        CodeToken::Operator => Face::color(Ansi::Red),
        CodeToken::Punctuation => Face::color(Ansi::Gray),
        CodeToken::Tag => Face::color(Ansi::Blue),
    }
}

pub fn style_for_name(name: &str) -> Option<Style> {
    let token = match name {
        "keyword" => CodeToken::Keyword,
        "string" => CodeToken::String,
        "number" => CodeToken::Number,
        "comment" => CodeToken::Comment,
        "function" => CodeToken::Function,
        "type" => CodeToken::Type,
        "constant" => CodeToken::Constant,
        "variable" => CodeToken::Variable,
        "operator" => CodeToken::Operator,
        "punctuation" => CodeToken::Punctuation,
        "property" => CodeToken::Property,
        "attribute" => CodeToken::Attribute,
        _ => return markdown_style_for_name(name),
    };
    Some(Style::Code(token))
}

fn markdown_style_for_name(name: &str) -> Option<Style> {
    Some(match name {
        "text" => Style::Text,
        "heading1" => Style::Heading1,
        "heading2" => Style::Heading2,
        "heading3" => Style::Heading3,
        "bold" => Style::Bold,
        "italic" => Style::Italic,
        "codespan" => Style::CodeSpan,
        "codeblock" => Style::CodeBlock,
        "frame" => Style::Frame,
        "link" => Style::Link,
        "hashtag" => Style::Tag,
        "quote" => Style::Quote,
        "frontmatter" => Style::Frontmatter,
        _ => return None,
    })
}
