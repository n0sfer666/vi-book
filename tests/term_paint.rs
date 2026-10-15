use v_ide_tui::term_paint::paint;
use v_ide_view::{Ansi, Color, Depth, Mods, Op, Style};

fn bytes(ops: &[Op], depth: Depth) -> String {
    let mut out = Vec::new();
    paint(&mut out, ops, depth).unwrap();
    String::from_utf8(out).unwrap()
}

fn fg(color: Color, depth: Depth) -> String {
    let style = Style {
        fg: color,
        ..Style::default()
    };
    bytes(&[Op::Style(style)], depth)
}

#[test]
fn truecolor_keeps_rgb() {
    let out = fg(Color::Rgb(255, 0, 0), Depth::TrueColor);
    assert!(out.contains("\x1b[38;2;255;0;0m"), "{out:?}");
}

#[test]
fn ansi256_downsamples_rgb_to_index() {
    let out = fg(Color::Rgb(255, 0, 0), Depth::Ansi256);
    assert!(out.contains("\x1b[38;5;196m"), "{out:?}");
    assert!(!out.contains("38;2;"), "{out:?}");
}

#[test]
fn ansi16_downsamples_rgb_and_indexed_to_named_slots() {
    assert!(fg(Color::Rgb(255, 0, 0), Depth::Ansi16).contains("\x1b[38;5;9m"));
    assert!(fg(Color::Indexed(196), Depth::Ansi16).contains("\x1b[38;5;9m"));
}

#[test]
fn named_colors_map_to_terminal_slots() {
    for (ansi, slot) in [
        (Ansi::Red, 1),
        (Ansi::Gray, 7),
        (Ansi::DarkGray, 8),
        (Ansi::LightRed, 9),
        (Ansi::White, 15),
    ] {
        let out = fg(Color::Ansi(ansi), Depth::TrueColor);
        assert!(
            out.contains(&format!("\x1b[38;5;{slot}m")),
            "{ansi:?}: {out:?}"
        );
    }
}

#[test]
fn background_is_painted() {
    let style = Style {
        bg: Color::Indexed(42),
        ..Style::default()
    };
    let out = bytes(&[Op::Style(style)], Depth::Ansi256);
    assert!(out.contains("\x1b[48;5;42m"), "{out:?}");
}

#[test]
fn each_style_starts_from_reset_so_mods_do_not_leak() {
    let out = bytes(
        &[
            Op::Style(Style::default().bold()),
            Op::Style(Style::default()),
        ],
        Depth::TrueColor,
    );
    let second = out.rfind("\x1b[0m").unwrap();
    assert!(out.find("\x1b[1m").unwrap() < second, "{out:?}");
    assert!(!out[second..].contains("\x1b[1m"), "{out:?}");
}

#[test]
fn mods_map_to_attributes_and_undercurl_is_underline() {
    let style = Style {
        mods: Mods {
            bold: true,
            italic: true,
            undercurl: true,
            dim: true,
            reverse: true,
            strikethrough: true,
            ..Mods::default()
        },
        ..Style::default()
    };
    let out = bytes(&[Op::Style(style)], Depth::TrueColor);
    for sgr in ["1", "2", "3", "4", "7", "9"] {
        assert!(out.contains(&format!("\x1b[{sgr}m")), "{sgr}: {out:?}");
    }
}

#[test]
fn move_and_text_are_written_in_order() {
    let out = bytes(
        &[Op::Move { x: 2, y: 1 }, Op::Text(String::from("ab"))],
        Depth::TrueColor,
    );
    assert_eq!(out, "\x1b[2;3Hab");
}

#[test]
fn clear_resets_style_before_clearing() {
    let out = bytes(&[Op::Clear], Depth::TrueColor);
    let reset = out.find("\x1b[0m").unwrap();
    assert!(reset < out.find("\x1b[2J").unwrap(), "{out:?}");
}
