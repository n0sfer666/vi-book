use crate::style::Style;
use crate::symbol::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub symbol: Symbol,
    pub width: u8,
    pub style: Style,
}

impl Cell {
    pub(crate) fn blank(style: Style) -> Self {
        Self {
            symbol: Symbol::new(" "),
            width: 1,
            style,
        }
    }

    pub(crate) fn continuation(style: Style) -> Self {
        Self {
            symbol: Symbol::new(""),
            width: 0,
            style,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank(Style::default())
    }
}
