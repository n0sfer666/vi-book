use v_ide_markdown::style::Style;
use v_ide_tui::theme::{Face, Theme};
use v_ide_view::{Ansi, Color};

#[test]
fn default_theme_covers_every_style() {
    let theme = Theme::default();
    for style in Style::ALL {
        let _ = theme.face(style);
    }
    assert_eq!(Style::ALL.len(), 15);
}

#[test]
fn links_are_underlined_and_colored() {
    let face = Theme::default().face(Style::Link);
    assert!(face.underline);
    assert_eq!(face.color, Some(Color::Ansi(Ansi::Blue)));
}

#[test]
fn headings_are_bold() {
    let theme = Theme::default();
    for style in [Style::Heading1, Style::Heading2, Style::Heading3] {
        assert!(theme.face(style).bold, "{style:?} should be bold");
    }
}

#[test]
fn code_styles_have_color() {
    let theme = Theme::default();
    assert!(theme.face(Style::CodeSpan).color.is_some());
    assert!(theme.face(Style::CodeBlock).color.is_some());
}

#[test]
fn structural_styles_are_dim() {
    let theme = Theme::default();
    for style in [
        Style::Quote,
        Style::QuoteBar,
        Style::Frame,
        Style::Frontmatter,
    ] {
        assert!(theme.face(style).dim, "{style:?} should be dim");
    }
}

#[test]
fn plain_text_has_no_decoration() {
    assert_eq!(Theme::default().face(Style::Text), Face::default());
}

#[test]
fn face_converts_to_view_style() {
    let face = Face {
        color: Some(Color::Ansi(Ansi::Red)),
        bold: true,
        underline: true,
        ..Face::default()
    };
    let style = face.style();
    assert_eq!(style.fg, Color::Ansi(Ansi::Red));
    assert_eq!(style.bg, Color::Reset);
    assert!(style.mods.bold);
    assert!(style.mods.underline);
    assert!(!style.mods.italic);
    assert_eq!(Face::default().style().fg, Color::Reset);
}

#[test]
fn theme_set_overrides_a_style() {
    let mut theme = Theme::default();
    let custom = Face {
        color: Some(Color::Ansi(Ansi::Green)),
        ..Face::default()
    };
    theme.set(Style::Tag, custom);
    assert_eq!(theme.face(Style::Tag).color, Some(Color::Ansi(Ansi::Green)));
}
