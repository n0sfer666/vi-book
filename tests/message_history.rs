mod common;

use common::feed::app;
use v_ide_editor::module::Request;
use v_ide_tui::{command_line, requests};

#[test]
fn module_messages_preserve_order_repeats_and_last_status() {
    let mut app = app("text");
    requests::apply(
        &mut app,
        vec![
            Request::Message("first".into()),
            Request::Message("first".into()),
            Request::Message("last".into()),
        ],
    );
    assert_eq!(app.ui.message, "last");
    assert_eq!(
        app.ui.messages.history().collect::<Vec<_>>(),
        vec!["first", "first", "last"]
    );
    assert_eq!(app.ui.pager.as_ref().unwrap().text(), "first\nfirst\nlast");
}

#[test]
fn history_keeps_last_thousand_nonempty_messages() {
    let mut app = app("text");
    let messages = (0..1002)
        .map(|n| Request::Message(format!("message {n}")))
        .collect();
    requests::apply(&mut app, messages);
    requests::apply(&mut app, vec![Request::Message(String::new())]);
    let history = app.ui.messages.history().collect::<Vec<_>>();
    assert_eq!(history.len(), 1000);
    assert_eq!(history.first(), Some(&"message 2"));
    assert_eq!(history.last(), Some(&"message 1001"));
}

#[test]
fn messages_outputs_history_without_recording_it_again() {
    let mut app = app("text");
    requests::apply(&mut app, vec![Request::Message("a".into())]);
    requests::apply(&mut app, vec![Request::Message("b".into())]);
    command_line::run(&mut app, "messages");
    assert_eq!(app.ui.pager.as_ref().unwrap().text(), "a\nb");
    assert_eq!(
        app.ui.messages.history().collect::<Vec<_>>(),
        vec!["a", "b"]
    );
}

#[test]
fn editor_errors_also_reach_history() {
    let mut app = app("text");
    command_line::run(&mut app, "missing");
    command_line::run(&mut app, "w");
    let history = app.ui.messages.history().collect::<Vec<_>>();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0], "not an editor command: missing");
    assert!(history[1].starts_with("save failed:"));
}

#[test]
fn short_message_stays_on_status_and_background_output_keeps_open_pager_stable() {
    let mut app = app("text");
    requests::apply(&mut app, vec![Request::Message("short".into())]);
    assert!(app.ui.pager.is_none());
    requests::apply(&mut app, vec![Request::Message("first\nsecond".into())]);
    requests::apply(&mut app, vec![Request::Message("background".into())]);
    assert_eq!(app.ui.pager.as_ref().unwrap().text(), "first\nsecond");
    assert_eq!(app.ui.messages.history().last(), Some("background"));
}

#[test]
fn messages_clear_removes_history_without_creating_a_new_entry() {
    let mut app = app("text");
    requests::apply(&mut app, vec![Request::Message("old".into())]);
    command_line::run(&mut app, "messages clear");
    assert!(app.ui.messages.history().next().is_none());
    assert_eq!(app.ui.message, "");
    command_line::run(&mut app, "messages");
    assert!(app.ui.pager.is_none());
    assert!(app.ui.messages.history().next().is_none());
}
