use v_ide_engine::editor::Editor;
use v_ide_markdown::render::{render, render_with};
use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_tui::app::App;
use v_ide_tui::read_view::cached_render;

#[test]
fn cached_render_matches_direct_render() {
    let mut app = App::new(Editor::new("# Title\n\ntext"), "a.md".into());
    let cached = cached_render(&mut app, 40);
    assert_eq!(
        cached.text(),
        render_with("# Title\n\ntext", 40, WrapMode::All).text()
    );
}

#[test]
fn cache_reflects_buffer_changes() {
    let mut app = App::new(Editor::new("# Title"), "a.md".into());
    let first = cached_render(&mut app, 40);
    assert_eq!(
        first.text(),
        render_with("# Title", 40, WrapMode::All).text()
    );

    app.editor.replace_all("## Other");
    let second = cached_render(&mut app, 40);
    assert_eq!(
        second.text(),
        render_with("## Other", 40, WrapMode::All).text()
    );
}

#[test]
fn cache_reflects_width_changes() {
    let mut app = App::new(Editor::new("aaa bbb ccc"), "a.md".into());
    app.wrap = WrapMode::All;
    let narrow = cached_render(&mut app, 7);
    let wide = cached_render(&mut app, 40);
    assert_eq!(narrow.text(), render("aaa bbb ccc", 7).text());
    assert_eq!(wide.text(), render("aaa bbb ccc", 40).text());
}

#[test]
fn cache_reflects_wrap_mode_changes() {
    let mut app = App::new(Editor::new("aaa bbb ccc"), "a.md".into());
    app.wrap = WrapMode::None;
    let none = cached_render(&mut app, 7).text();
    app.wrap = WrapMode::All;
    let all = cached_render(&mut app, 7).text();
    assert_ne!(none, all, "changing wrap mode invalidates the cache");
}
