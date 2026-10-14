use v_ide_view::{Ansi, Color, Depth};

fn parse(s: &str) -> Option<Color> {
    s.parse::<Color>().ok()
}

#[test]
fn names_parse_like_ratatui() {
    assert_eq!(parse("red"), Some(Color::Ansi(Ansi::Red)));
    assert_eq!(parse("Light-Blue"), Some(Color::Ansi(Ansi::LightBlue)));
    assert_eq!(parse("bright_red"), Some(Color::Ansi(Ansi::LightRed)));
    assert_eq!(parse("grey"), Some(Color::Ansi(Ansi::Gray)));
    assert_eq!(parse("silver"), Some(Color::Ansi(Ansi::Gray)));
    assert_eq!(parse("dark gray"), Some(Color::Ansi(Ansi::DarkGray)));
    assert_eq!(parse("lightgray"), Some(Color::Ansi(Ansi::White)));
    assert_eq!(parse("lightblack"), Some(Color::Ansi(Ansi::DarkGray)));
    assert_eq!(parse("reset"), Some(Color::Reset));
}

#[test]
fn index_and_hex_parse() {
    assert_eq!(parse("42"), Some(Color::Indexed(42)));
    assert_eq!(parse("#ff8000"), Some(Color::Rgb(255, 128, 0)));
    assert_eq!(parse("#12345"), None);
    assert_eq!(parse("nonsense"), None);
    assert_eq!(parse("256"), None);
}

#[test]
fn depth_is_detected_from_env() {
    assert_eq!(
        Depth::detect(Some("truecolor"), Some("xterm")),
        Depth::TrueColor
    );
    assert_eq!(Depth::detect(Some("24bit"), None), Depth::TrueColor);
    assert_eq!(Depth::detect(None, Some("xterm-256color")), Depth::Ansi256);
    assert_eq!(Depth::detect(Some("yes"), Some("xterm")), Depth::Ansi16);
    assert_eq!(Depth::detect(None, None), Depth::Ansi16);
}

#[test]
fn truecolor_keeps_everything() {
    for c in [Color::Rgb(1, 2, 3), Color::Indexed(200), Color::Reset] {
        assert_eq!(c.downsample(Depth::TrueColor), c);
    }
}

#[test]
fn rgb_maps_to_nearest_of_256() {
    assert_eq!(
        Color::Rgb(255, 0, 0).downsample(Depth::Ansi256),
        Color::Indexed(196)
    );
    assert_eq!(
        Color::Rgb(128, 128, 128).downsample(Depth::Ansi256),
        Color::Indexed(244)
    );
    assert_eq!(
        Color::Indexed(77).downsample(Depth::Ansi256),
        Color::Indexed(77)
    );
}

#[test]
fn rgb_and_index_map_to_nearest_of_16() {
    let red = Color::Ansi(Ansi::LightRed);
    assert_eq!(Color::Rgb(255, 0, 0).downsample(Depth::Ansi16), red);
    assert_eq!(Color::Indexed(196).downsample(Depth::Ansi16), red);
    assert_eq!(
        Color::Rgb(10, 10, 10).downsample(Depth::Ansi16),
        Color::Ansi(Ansi::Black)
    );
    assert_eq!(
        Color::Indexed(3).downsample(Depth::Ansi16),
        Color::Ansi(Ansi::Yellow)
    );
}

#[test]
fn ansi_and_reset_survive_any_depth() {
    for depth in [Depth::Ansi16, Depth::Ansi256, Depth::TrueColor] {
        assert_eq!(Color::Reset.downsample(depth), Color::Reset);
        assert_eq!(
            Color::Ansi(Ansi::Cyan).downsample(depth),
            Color::Ansi(Ansi::Cyan)
        );
    }
}
