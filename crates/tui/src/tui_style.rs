use ratatui::style::{Color as TuiColor, Modifier, Style as TuiStyle};
use v_ide_view::{Ansi, Color, Style};

pub(crate) fn from_cell(style: Style) -> TuiStyle {
    let mut tui = TuiStyle::default();
    if let Some(fg) = color(style.fg) {
        tui = tui.fg(fg);
    }
    if let Some(bg) = color(style.bg) {
        tui = tui.bg(bg);
    }
    let mods = style.mods;
    for (on, modifier) in [
        (mods.bold, Modifier::BOLD),
        (mods.italic, Modifier::ITALIC),
        (mods.underline || mods.undercurl, Modifier::UNDERLINED),
        (mods.dim, Modifier::DIM),
        (mods.reverse, Modifier::REVERSED),
        (mods.strikethrough, Modifier::CROSSED_OUT),
    ] {
        if on {
            tui = tui.add_modifier(modifier);
        }
    }
    tui
}

fn color(color: Color) -> Option<TuiColor> {
    match color {
        Color::Reset => None,
        Color::Ansi(ansi) => Some(named(ansi)),
        Color::Indexed(index) => Some(TuiColor::Indexed(index)),
        Color::Rgb(r, g, b) => Some(TuiColor::Rgb(r, g, b)),
    }
}

fn named(ansi: Ansi) -> TuiColor {
    match ansi {
        Ansi::Black => TuiColor::Black,
        Ansi::Red => TuiColor::Red,
        Ansi::Green => TuiColor::Green,
        Ansi::Yellow => TuiColor::Yellow,
        Ansi::Blue => TuiColor::Blue,
        Ansi::Magenta => TuiColor::Magenta,
        Ansi::Cyan => TuiColor::Cyan,
        Ansi::Gray => TuiColor::Gray,
        Ansi::DarkGray => TuiColor::DarkGray,
        Ansi::LightRed => TuiColor::LightRed,
        Ansi::LightGreen => TuiColor::LightGreen,
        Ansi::LightYellow => TuiColor::LightYellow,
        Ansi::LightBlue => TuiColor::LightBlue,
        Ansi::LightMagenta => TuiColor::LightMagenta,
        Ansi::LightCyan => TuiColor::LightCyan,
        Ansi::White => TuiColor::White,
    }
}
