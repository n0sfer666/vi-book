mod common;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

use v_ide_tui::app::App;
use v_ide_tui::split::split_rects;
use v_ide_tui::view::View;

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
    let mut app = App::new(common::named_doc("alpha", "a.md"));
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
    let mut app = App::new(common::named_doc("alpha", "a.md"));
    app.vsplit(None);
    app.workspace
        .current_mut()
        .editor
        .handle_key(v_ide_keys::Key::Char('x'));
    let active_text = app.workspace.current().editor.text();
    app.focus_next();
    assert_ne!(
        app.workspace.current().editor.text(),
        active_text,
        "other pane is independent"
    );
    assert_eq!(app.workspace.current().editor.text(), "alpha");
}

#[test]
fn focus_next_cycles_through_panes() {
    let mut app = App::new(common::named_doc("one", "a.md"));
    app.vsplit(None);
    assert_eq!(app.workspace.active_index(), 1);
    app.focus_next();
    assert_eq!(app.workspace.active_index(), 0);
    app.focus_next();
    assert_eq!(app.workspace.active_index(), 1);
}

#[test]
fn two_panes_render_side_by_side_with_a_divider() {
    let mut app = App::new(common::named_doc("AAAA", "a.md"));
    app.ui.read.view = View::Edit;
    app.vsplit(None);
    app.ui.read.view = View::Edit;
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| v_ide_tui::render::draw(frame, &mut app))
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
    let mut app = App::new(common::named_doc("one", "a.md"));
    app.vsplit(None);
    app.vsplit(None);
    assert_eq!(app.pane_count(), 3);
    app.only();
    assert_eq!(app.pane_count(), 1);
    assert_eq!(app.workspace.active_index(), 0);
}

fn app_with_dirty_split() -> App {
    let mut app = App::new(common::named_doc("alpha", "a.md"));
    app.vsplit(None);
    app.workspace
        .current_mut()
        .editor
        .handle_key(v_ide_keys::Key::Char('x'));
    app
}

#[test]
fn quit_refuses_when_an_inactive_window_is_dirty() {
    let mut app = app_with_dirty_split();
    app.focus_next();
    v_ide_tui::command_line::run(&mut app, "q");
    assert!(!app.quit, "dirty split window blocks :q");
    assert!(app.ui.message.contains("unsaved"));
}

#[test]
fn close_refuses_a_dirty_window() {
    let mut app = app_with_dirty_split();
    v_ide_tui::command_line::run(&mut app, "close");
    assert_eq!(app.pane_count(), 2, "dirty window stays open");
    assert!(app.ui.message.contains("unsaved"));
}

#[test]
fn only_refuses_when_another_window_is_dirty() {
    let mut app = app_with_dirty_split();
    app.focus_next();
    v_ide_tui::command_line::run(&mut app, "only");
    assert_eq!(app.pane_count(), 2, "dirty window is not discarded");
    assert!(app.ui.message.contains("unsaved"));
}
