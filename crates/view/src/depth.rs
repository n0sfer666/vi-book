#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Depth {
    Ansi16,
    Ansi256,
    TrueColor,
}

impl Depth {
    pub fn detect(colorterm: Option<&str>, term: Option<&str>) -> Self {
        if matches!(colorterm, Some("truecolor" | "24bit")) {
            return Self::TrueColor;
        }
        if term.is_some_and(|term| term.contains("256color")) {
            return Self::Ansi256;
        }
        Self::Ansi16
    }
}
