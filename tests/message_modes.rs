mod common;

use common::feed::{app, ctrl, press};
use v_ide_editor::module::Request;
use v_ide_engine::Mode;
use v_ide_tui::{command_line::run, requests};

#[test]
fn colon_at_more_leaves_insert_and_discards_the_interrupted_mapping() {
    let mut app = app("one\ntwo\nthree");
    common::draw(&mut app, 40, 7);
    for n in 0..20 {
        requests::apply(&mut app, vec![Request::Message(format!("message {n}"))]);
    }
    run(&mut app, "inoremap <C-s> <Cmd>messages<CR>j<Cmd>q!<CR>");
    press(&mut app, "i");
    ctrl(&mut app, 's');
    assert!(app.ui.pager.as_ref().unwrap().more());
    press(&mut app, ":");
    assert!(app.ui.pager.is_none());
    assert!(app.ui.command.is_none());
    assert!(!app.quit);
    assert_eq!(app.workspace.current().editor.mode(), Mode::Normal);
    assert_eq!(app.workspace.current().editor.cursor().line, 0);
    press(&mut app, "l");
    assert_eq!(app.workspace.current().editor.cursor().col, 1);
    assert_eq!(app.workspace.current().editor.text(), "one\ntwo\nthree");
}

#[test]
fn messages_during_search_do_not_steal_search_submission() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut app = app("one\ntwo\nthree");
    press(&mut app, "/two");
    requests::apply(&mut app, vec![Request::Message("first\nsecond".into())]);
    assert!(app.ui.pager.is_none());
    app.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let editor = &app.workspace.current().editor;
    assert!(editor.search_input().is_none());
    assert_eq!(editor.cursor().line, 1);
}

#[test]
fn colon_at_more_leaves_visual_and_opens_range_without_the_mapping_tail() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    for visual in ["v", "V"] {
        let mut app = app("one\ntwo\nthree");
        common::draw(&mut app, 40, 7);
        for n in 0..20 {
            requests::apply(&mut app, vec![Request::Message(format!("message {n}"))]);
        }
        run(&mut app, "xnoremap <C-s> <Cmd>messages<CR>j<Cmd>q!<CR>");
        press(&mut app, visual);
        ctrl(&mut app, 's');
        assert!(app.ui.pager.as_ref().unwrap().more());
        press(&mut app, ":");
        assert!(app.ui.pager.is_none());
        assert_eq!(app.workspace.current().editor.mode(), Mode::Normal);
        assert_eq!(app.ui.command.as_deref(), Some("'<,'>"));
        assert!(!app.quit);
        app.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.ui.command.is_none());
        assert_eq!(app.workspace.current().editor.mode(), Mode::Normal);
        assert_eq!(app.workspace.current().editor.text(), "one\ntwo\nthree");
    }
}

#[test]
fn colon_at_more_cancels_pending_operator_but_preserves_the_mapping_tail() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut app = app("one\ntwo\nthree");
    common::draw(&mut app, 40, 7);
    for n in 0..20 {
        requests::apply(&mut app, vec![Request::Message(format!("message {n}"))]);
    }
    run(
        &mut app,
        "onoremap <C-s> <Cmd>messages<CR>j<Cmd>nnoremap X l<CR>",
    );
    press(&mut app, "d");
    ctrl(&mut app, 's');
    assert!(app.ui.pager.as_ref().unwrap().more());
    press(&mut app, ":");
    assert!(app.ui.pager.is_none());
    assert!(app.workspace.current().editor.idle_normal());
    assert_eq!(app.ui.command.as_deref(), Some("j"));
    app.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    press(&mut app, "X");
    assert_eq!(app.workspace.current().editor.cursor().col, 1);
    assert_eq!(app.workspace.current().editor.text(), "one\ntwo\nthree");
}
