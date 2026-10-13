#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeToken {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Constant,
    Variable,
    Operator,
    Punctuation,
    Property,
    Tag,
    Attribute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    Text,
    Heading1,
    Heading2,
    Heading3,
    Bold,
    Italic,
    CodeSpan,
    CodeBlock,
    Frame,
    Link,
    Tag,
    Quote,
    QuoteBar,
    Frontmatter,
    ImagePlaceholder,
    Code(CodeToken),
}

impl Style {
    pub const ALL: [Style; 15] = [
        Style::Text,
        Style::Heading1,
        Style::Heading2,
        Style::Heading3,
        Style::Bold,
        Style::Italic,
        Style::CodeSpan,
        Style::CodeBlock,
        Style::Frame,
        Style::Link,
        Style::Tag,
        Style::Quote,
        Style::QuoteBar,
        Style::Frontmatter,
        Style::ImagePlaceholder,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: Style,
    pub source: Option<(usize, usize)>,
}

impl Span {
    pub fn decoration(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
            source: None,
        }
    }

    pub fn mapped(text: impl Into<String>, style: Style, start: usize, end: usize) -> Self {
        Self {
            text: text.into(),
            style,
            source: Some((start, end)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}
