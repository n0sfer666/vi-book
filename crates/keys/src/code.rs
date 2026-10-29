use crate::fkey::FKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code {
    Char(char),
    Esc,
    Enter,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    F(FKey),
}
