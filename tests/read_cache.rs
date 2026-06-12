use vi_book::markdown::render::render;
use vi_book::ui::app::App;
use vi_book::ui::read_view::cached_render;
use vi_book::vim::editor::Editor;

#[test]
fn cached_render_matches_direct_render() {
    let mut app = App::new(Editor::new("# Title\n\ntext"), "a.md".into());
    let cached = cached_render(&mut app, 40);
    assert_eq!(cached.text(), render("# Title\n\ntext", 40).text());
}

#[test]
fn cache_reflects_buffer_changes() {
    let mut app = App::new(Editor::new("# Title"), "a.md".into());
    let first = cached_render(&mut app, 40);
    assert_eq!(first.text(), render("# Title", 40).text());

    app.editor.replace_all("## Other");
    let second = cached_render(&mut app, 40);
    assert_eq!(second.text(), render("## Other", 40).text());
}

#[test]
fn cache_reflects_width_changes() {
    let mut app = App::new(Editor::new("aaa bbb ccc"), "a.md".into());
    let narrow = cached_render(&mut app, 7);
    let wide = cached_render(&mut app, 40);
    assert_eq!(narrow.text(), render("aaa bbb ccc", 7).text());
    assert_eq!(wide.text(), render("aaa bbb ccc", 40).text());
}
