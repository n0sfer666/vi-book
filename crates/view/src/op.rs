use crate::style::Style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Clear,
    Move { x: u16, y: u16 },
    Style(Style),
    Text(String),
}
