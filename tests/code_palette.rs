mod common;

use v_ide_markdown::style::{CodeToken, Style};
use v_ide_tui::app::App;
use v_ide_tui::theme::{Face, Theme, style_for_name};
use v_ide_tui::view::View;
use v_ide_view::{Ansi, Color};

#[test]
fn code_tokens_have_default_colors() {
    let theme = Theme::default();
    for token in [
        CodeToken::Keyword,
        CodeToken::String,
        CodeToken::Number,
        CodeToken::Comment,
        CodeToken::Function,
        CodeToken::Type,
    ] {
        assert!(
            theme.face(Style::Code(token)).color.is_some(),
            "{token:?} should have a default color"
        );
    }
}

#[test]
fn code_palette_is_overridable() {
    let mut theme = Theme::default();
    theme.set(
        Style::Code(CodeToken::Keyword),
        Face {
            color: Some(Color::Ansi(Ansi::Red)),
            ..Face::default()
        },
    );
    assert_eq!(
        theme.face(Style::Code(CodeToken::Keyword)).color,
        Some(Color::Ansi(Ansi::Red))
    );
}

#[test]
fn rendered_code_block_paints_keyword_color() {
    let mut app = App::new(common::named_doc(
        "```python\ndef f():\n    pass\n```",
        "a.md",
    ));
    app.ui.read.view = View::Read;
    let grid = common::draw(&mut app, 60, 24).grid;
    let magenta = (0..grid.w)
        .any(|x| (0..grid.h).any(|y| grid.cell(x, y).style.fg == Color::Ansi(Ansi::Magenta)));
    assert!(magenta, "keyword should be painted with its palette color");
}

#[test]
fn names_map_to_code_and_markdown_styles() {
    assert_eq!(
        style_for_name("keyword"),
        Some(Style::Code(CodeToken::Keyword))
    );
    assert_eq!(
        style_for_name("comment"),
        Some(Style::Code(CodeToken::Comment))
    );
    assert_eq!(style_for_name("heading1"), Some(Style::Heading1));
    assert_eq!(style_for_name("link"), Some(Style::Link));
    assert_eq!(style_for_name("nonsense"), None);
}
