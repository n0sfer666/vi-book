use v_ide_view::{Ansi, Color, Mods, Style};

#[test]
fn reverse_turns_on_only_reverse() {
    let style = Style::default().reverse();
    assert_eq!(
        style.mods,
        Mods {
            reverse: true,
            ..Mods::default()
        }
    );
}

#[test]
fn dim_turns_on_only_dim() {
    let style = Style::default().dim();
    assert_eq!(
        style.mods,
        Mods {
            dim: true,
            ..Mods::default()
        }
    );
}

#[test]
fn bold_turns_on_only_bold() {
    let style = Style::default().bold();
    assert_eq!(
        style.mods,
        Mods {
            bold: true,
            ..Mods::default()
        }
    );
}

#[test]
fn chaining_keeps_colors_and_existing_mods() {
    let base = Style {
        fg: Color::Ansi(Ansi::Red),
        bg: Color::Indexed(17),
        ..Style::default()
    }
    .bold();
    let style = base.reverse();
    assert_eq!(style.fg, Color::Ansi(Ansi::Red));
    assert_eq!(style.bg, Color::Indexed(17));
    assert!(style.mods.bold && style.mods.reverse);
}
