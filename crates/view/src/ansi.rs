#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ansi {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

impl Ansi {
    pub(crate) const ALL: [Self; 16] = [
        Self::Black,
        Self::Red,
        Self::Green,
        Self::Yellow,
        Self::Blue,
        Self::Magenta,
        Self::Cyan,
        Self::Gray,
        Self::DarkGray,
        Self::LightRed,
        Self::LightGreen,
        Self::LightYellow,
        Self::LightBlue,
        Self::LightMagenta,
        Self::LightCyan,
        Self::White,
    ];

    pub(crate) fn from_name(name: &str) -> Option<Self> {
        let ansi = match name {
            "black" => Self::Black,
            "red" => Self::Red,
            "green" => Self::Green,
            "yellow" => Self::Yellow,
            "blue" => Self::Blue,
            "magenta" => Self::Magenta,
            "cyan" => Self::Cyan,
            "gray" => Self::Gray,
            "darkgray" => Self::DarkGray,
            "lightred" => Self::LightRed,
            "lightgreen" => Self::LightGreen,
            "lightyellow" => Self::LightYellow,
            "lightblue" => Self::LightBlue,
            "lightmagenta" => Self::LightMagenta,
            "lightcyan" => Self::LightCyan,
            "white" => Self::White,
            _ => return None,
        };
        Some(ansi)
    }

    pub(crate) fn rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Black => (0, 0, 0),
            Self::Red => (205, 0, 0),
            Self::Green => (0, 205, 0),
            Self::Yellow => (205, 205, 0),
            Self::Blue => (0, 0, 238),
            Self::Magenta => (205, 0, 205),
            Self::Cyan => (0, 205, 205),
            Self::Gray => (229, 229, 229),
            Self::DarkGray => (127, 127, 127),
            Self::LightRed => (255, 0, 0),
            Self::LightGreen => (0, 255, 0),
            Self::LightYellow => (255, 255, 0),
            Self::LightBlue => (92, 92, 255),
            Self::LightMagenta => (255, 0, 255),
            Self::LightCyan => (0, 255, 255),
            Self::White => (255, 255, 255),
        }
    }
}
