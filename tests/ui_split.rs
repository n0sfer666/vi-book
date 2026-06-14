use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

use vi_book::Editor;
use vi_book::ui::app::App;
use vi_book::ui::split::split_rects;
use vi_book::ui::view::View;

#[test]
fn one_pane_fills_the_area() {
    let area = Rect::new(0, 0, 80, 24);
    let rects = split_rects(area, 1);
    assert_eq!(rects, vec![area]);
}

#[test]
fn two_panes_share_width_with_a_gap() {
    let rects = split_rects(Rect::new(0, 0, 81, 24), 2);
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].x, 0);
    assert_eq!(rects[0].width, 40);
    assert_eq!(rects[1].x, 41);
    assert_eq!(rects[1].width, 40);
}

#[test]
fn three_panes_fit_within_the_area() {
    let area = Rect::new(0, 0, 80, 24);
    let rects = split_rects(area, 3);
    assert_eq!(rects.len(), 3);
    let last = rects.last().unwrap();
    assert!(last.x + last.width <= area.x + area.width);
    for pair in rects.windows(2) {
        assert!(pair[1].x > pair[0].x + pair[0].width);
    }
}

#[test]
fn vsplit_adds_independent_panes_up_to_three() {
    let mut app = App::new(Editor::new("alpha"), "a.md".into());
    assert_eq!(app.pane_count(), 1);
    app.vsplit(None);
    assert_eq!(app.pane_count(), 2);
    app.vsplit(None);
    assert_eq!(app.pane_count(), 3);
    app.vsplit(None);
    assert_eq!(app.pane_count(), 3, "capped at three columns");
}

#[test]
fn editing_active_pane_does_not_touch_others() {
    let mut app = App::new(Editor::new("alpha"), "a.md".into());
    app.vsplit(None);
    app.editor.handle_key(vi_book::Key::Char('x'));
    let active_text = app.editor.text();
    app.focus_next();
    assert_ne!(app.editor.text(), active_text, "other pane is independent");
    assert_eq!(app.editor.text(), "alpha");
}

#[test]
fn focus_next_cycles_through_panes() {
    let mut app = App::new(Editor::new("one"), "a.md".into());
    app.vsplit(None);
    app.active = 0;
    app.focus_next();
    assert_eq!(app.active, 1);
    app.focus_next();
    assert_eq!(app.active, 0);
}

#[test]
fn two_panes_render_side_by_side_with_a_divider() {
    let mut app = App::new(Editor::new("AAAA"), "a.md".into());
    app.view = View::Edit;
    app.vsplit(None);
    app.view = View::Edit;
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| vi_book::ui::render::draw(frame, &mut app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let row0: Vec<String> = (0..80).map(|x| buf[(x, 0)].symbol().to_string()).collect();
    let left = row0[..40].iter().any(|c| c == "A");
    let right = row0[40..].iter().any(|c| c == "A");
    let divider = row0.iter().any(|c| c == "\u{2502}");
    assert!(left && right, "both panes should show content");
    assert!(divider, "a divider should separate the panes");
}

#[test]
fn only_collapses_to_a_single_pane() {
    let mut app = App::new(Editor::new("one"), "a.md".into());
    app.vsplit(None);
    app.vsplit(None);
    assert_eq!(app.pane_count(), 3);
    app.only();
    assert_eq!(app.pane_count(), 1);
    assert_eq!(app.active, 0);
}
