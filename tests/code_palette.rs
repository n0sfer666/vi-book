use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;

use v_ide::Editor;
use v_ide::markdown::style::{CodeToken, Style};
use v_ide::ui::app::App;
use v_ide::ui::theme::{Face, Theme, style_for_name};
use v_ide::ui::view::View;

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
            color: Some(Color::Red),
            ..Face::default()
        },
    );
    assert_eq!(
        theme.face(Style::Code(CodeToken::Keyword)).color,
        Some(Color::Red)
    );
}

#[test]
fn rendered_code_block_paints_keyword_color() {
    let mut app = App::new(
        Editor::new("```python\ndef f():\n    pass\n```"),
        "a.md".into(),
    );
    app.view = View::Read;
    let backend = TestBackend::new(60, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| v_ide::ui::render::draw(frame, &mut app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let magenta = (0..buf.area.width)
        .any(|x| (0..buf.area.height).any(|y| buf[(x, y)].style().fg == Some(Color::Magenta)));
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
