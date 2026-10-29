use std::fmt;

use crate::code::Code;
use crate::key::Key;
use crate::mods::Mods;
use crate::names::name_of;

pub fn format_keys(keys: &[Key]) -> String {
    keys.iter().map(Key::to_string).collect()
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.mods(), self.code()) {
            (Mods::NONE, Code::Char('<')) => f.write_str("<lt>"),
            (Mods::NONE, Code::Char(' ')) => f.write_str("<Space>"),
            (Mods::NONE, Code::Char(c)) => write!(f, "{c}"),
            (mods, code) => write!(f, "<{}{}>", prefix(mods), part(code, mods)),
        }
    }
}

fn prefix(mods: Mods) -> String {
    [
        (mods.alt, "M-"),
        (mods.ctrl, "C-"),
        (mods.shift, "S-"),
        (mods.sup, "D-"),
    ]
    .iter()
    .filter(|(on, _)| *on)
    .map(|(_, text)| *text)
    .collect()
}

fn part(code: Code, mods: Mods) -> String {
    match code {
        Code::Char('\\') if mods == Mods::CTRL => "\\".to_string(),
        Code::Char(c) if !matches!(c, '<' | ' ' | '|' | '\\') => {
            if mods.ctrl {
                c.to_ascii_uppercase().to_string()
            } else {
                c.to_string()
            }
        }
        _ => name_of(code).unwrap_or_default(),
    }
}
