use std::str::FromStr;

use crate::ansi::Ansi;
use crate::depth::Depth;
use crate::palette;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Color {
    #[default]
    Reset,
    Ansi(Ansi),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    pub fn downsample(self, depth: Depth) -> Self {
        match (self, depth) {
            (Self::Rgb(r, g, b), Depth::Ansi256) => Self::Indexed(palette::nearest_256((r, g, b))),
            (Self::Rgb(r, g, b), Depth::Ansi16) => Self::Ansi(palette::nearest_16((r, g, b))),
            (Self::Indexed(index), Depth::Ansi16) => {
                Self::Ansi(palette::nearest_16(palette::index_rgb(index)))
            }
            _ => self,
        }
    }
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let name = s
            .to_lowercase()
            .replace([' ', '-', '_'], "")
            .replace("bright", "light")
            .replace("grey", "gray")
            .replace("silver", "gray")
            .replace("lightblack", "darkgray")
            .replace("lightwhite", "white")
            .replace("lightgray", "white");
        if name == "reset" {
            return Ok(Self::Reset);
        }
        if let Some(ansi) = Ansi::from_name(&name) {
            return Ok(Self::Ansi(ansi));
        }
        if let Ok(index) = s.parse::<u8>() {
            return Ok(Self::Indexed(index));
        }
        hex(s).map(|(r, g, b)| Self::Rgb(r, g, b)).ok_or(())
    }
}

fn hex(s: &str) -> Option<(u8, u8, u8)> {
    if !s.starts_with('#') || s.len() != 7 {
        return None;
    }
    let part = |range| u8::from_str_radix(s.get(range)?, 16).ok();
    Some((part(1..3)?, part(3..5)?, part(5..7)?))
}
