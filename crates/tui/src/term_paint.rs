use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{
    Attribute, Color as TermColor, Print, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{Clear, ClearType};
use v_ide_view::{Ansi, Color, Depth, Op, Style};

pub fn paint(out: &mut impl Write, ops: &[Op], depth: Depth) -> io::Result<()> {
    for op in ops {
        match op {
            Op::Clear => queue!(out, SetAttribute(Attribute::Reset), Clear(ClearType::All))?,
            Op::Move { x, y } => queue!(out, MoveTo(*x, *y))?,
            Op::Style(style) => apply(out, *style, depth)?,
            Op::Text(text) => queue!(out, Print(text))?,
        }
    }
    Ok(())
}

fn apply(out: &mut impl Write, style: Style, depth: Depth) -> io::Result<()> {
    queue!(out, SetAttribute(Attribute::Reset))?;
    if let Some(fg) = color(style.fg.downsample(depth)) {
        queue!(out, SetForegroundColor(fg))?;
    }
    if let Some(bg) = color(style.bg.downsample(depth)) {
        queue!(out, SetBackgroundColor(bg))?;
    }
    let mods = style.mods;
    for (on, attribute) in [
        (mods.bold, Attribute::Bold),
        (mods.italic, Attribute::Italic),
        (mods.underline || mods.undercurl, Attribute::Underlined),
        (mods.dim, Attribute::Dim),
        (mods.reverse, Attribute::Reverse),
        (mods.strikethrough, Attribute::CrossedOut),
    ] {
        if on {
            queue!(out, SetAttribute(attribute))?;
        }
    }
    Ok(())
}

fn color(color: Color) -> Option<TermColor> {
    match color {
        Color::Reset => None,
        Color::Ansi(ansi) => Some(named(ansi)),
        Color::Indexed(index) => Some(TermColor::AnsiValue(index)),
        Color::Rgb(r, g, b) => Some(TermColor::Rgb { r, g, b }),
    }
}

fn named(ansi: Ansi) -> TermColor {
    match ansi {
        Ansi::Black => TermColor::Black,
        Ansi::Red => TermColor::DarkRed,
        Ansi::Green => TermColor::DarkGreen,
        Ansi::Yellow => TermColor::DarkYellow,
        Ansi::Blue => TermColor::DarkBlue,
        Ansi::Magenta => TermColor::DarkMagenta,
        Ansi::Cyan => TermColor::DarkCyan,
        Ansi::Gray => TermColor::Grey,
        Ansi::DarkGray => TermColor::DarkGrey,
        Ansi::LightRed => TermColor::Red,
        Ansi::LightGreen => TermColor::Green,
        Ansi::LightYellow => TermColor::Yellow,
        Ansi::LightBlue => TermColor::Blue,
        Ansi::LightMagenta => TermColor::Magenta,
        Ansi::LightCyan => TermColor::Cyan,
        Ansi::White => TermColor::White,
    }
}
