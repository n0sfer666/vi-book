use crate::color::Color;
use crate::mods::Mods;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub underline_color: Color,
    pub mods: Mods,
}

impl Style {
    pub fn reverse(mut self) -> Self {
        self.mods.reverse = true;
        self
    }

    pub fn dim(mut self) -> Self {
        self.mods.dim = true;
        self
    }

    pub fn bold(mut self) -> Self {
        self.mods.bold = true;
        self
    }
}
