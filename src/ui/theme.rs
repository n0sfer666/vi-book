use std::collections::HashMap;

use ratatui::style::{Color, Modifier, Style as TuiStyle};

use crate::markdown::style::{CodeToken, Style};

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
        Style::Code(token) => code_face(token),
    }
}

fn code_face(token: CodeToken) -> Face {
    match token {
        CodeToken::Keyword => Face::color(Color::Magenta),
        CodeToken::String => Face::color(Color::Green),
        CodeToken::Number | CodeToken::Constant | CodeToken::Attribute => {
            Face::color(Color::Yellow)
        }
        CodeToken::Comment => Face {
            color: Some(Color::DarkGray),
            dim: true,
            ..Face::default()
        },
        CodeToken::Function => Face::color(Color::Blue),
        CodeToken::Type | CodeToken::Property => Face::color(Color::Cyan),
        CodeToken::Variable => Face::color(Color::White),
        CodeToken::Operator => Face::color(Color::Red),
        CodeToken::Punctuation => Face::color(Color::Gray),
        CodeToken::Tag => Face::color(Color::Blue),
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
