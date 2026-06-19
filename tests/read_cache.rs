use vi_book::markdown::render::{render, render_with};
use vi_book::markdown::wrap_mode::WrapMode;
use vi_book::ui::app::App;
use vi_book::ui::read_view::cached_render;
use vi_book::vim::editor::Editor;

#[test]
fn cached_render_matches_direct_render() {
    let mut app = App::new(Editor::new("# Title\n\ntext"), "a.md".into());
    let cached = cached_render(&mut app, 40);
    assert_eq!(
        cached.text(),
        render_with("# Title\n\ntext", 40, WrapMode::None).text()
    );
}

#[test]
fn cache_reflects_buffer_changes() {
    let mut app = App::new(Editor::new("# Title"), "a.md".into());
    let first = cached_render(&mut app, 40);
    assert_eq!(
        first.text(),
        render_with("# Title", 40, WrapMode::None).text()
    );

    app.editor.replace_all("## Other");
    let second = cached_render(&mut app, 40);
    assert_eq!(
        second.text(),
        render_with("## Other", 40, WrapMode::None).text()
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
    let none = cached_render(&mut app, 7).text();
    app.wrap = WrapMode::All;
    let all = cached_render(&mut app, 7).text();
    assert_ne!(none, all, "changing wrap mode invalidates the cache");
}
