mod common;

use std::fs;
use v_ide_tui::{app::App, command_line};

fn app(label: &str) -> App {
    let root = common::scratch(label);
    for name in ["a.txt", "b.txt", "c.txt"] {
        fs::write(root.join(name), "one").unwrap();
    }
    let mut app = App::new(common::named_doc("", ""));
    app.workspace.set_cwd(root);
    app.open_rel("a.txt");
    app
}

fn run(app: &mut App, command: &str) {
    command_line::run(app, command);
}

fn query(app: &mut App, command: &str) -> String {
    run(app, command);
    app.ui.message.clone()
}

#[test]
fn window_global_defaults_are_copied_on_split_then_evolve_independently() {
    let mut app = app("set-window-defaults");
    app.vsplit(None);
    run(&mut app, "setglobal nu");
    assert_eq!(query(&mut app, "setglobal nu?"), "  number");
    assert_eq!(query(&mut app, "setlocal nu?"), "nonumber");
    app.focus_next();
    assert_eq!(query(&mut app, "setglobal nu?"), "nonumber");
    app.open_rel("b.txt");
    assert_eq!(query(&mut app, "setlocal nu?"), "nonumber");
    app.focus_next();
    app.open_rel("c.txt");
    assert_eq!(query(&mut app, "setlocal nu?"), "  number");
}

#[test]
fn first_visit_uses_options_from_another_live_or_closed_window() {
    for close in [false, true] {
        let mut app = app(if close {
            "set-closed-fallback"
        } else {
            "set-live-fallback"
        });
        app.vsplit(None);
        app.open_rel("b.txt");
        run(&mut app, "setlocal nu");
        if close {
            app.close_active();
        } else {
            app.focus_next();
        }
        app.open_rel("b.txt");
        assert_eq!(query(&mut app, "setlocal nu?"), "  number");
        assert_eq!(query(&mut app, "setglobal nu?"), "nonumber");
    }
}

#[test]
fn splitting_copies_current_values_but_does_not_clone_other_buffer_history() {
    let mut app = app("set-split-history");
    app.open_rel("b.txt");
    run(&mut app, "setlocal nonu");
    app.open_rel("a.txt");
    app.vsplit(None);
    app.open_rel("b.txt");
    run(&mut app, "setlocal nu");
    app.open_rel("a.txt");
    app.focus_next();
    app.vsplit(None);
    app.open_rel("b.txt");
    assert_eq!(query(&mut app, "setlocal nu?"), "  number");
}

#[test]
fn scoped_listings_omit_global_only_options_but_explicit_queries_allow_them() {
    let mut app = app("set-listing-scope");
    run(&mut app, "set tm=17 nu ro");
    for command in ["setlocal", "setlocal all", "setglobal", "setglobal all"] {
        let message = query(&mut app, command);
        assert!(message.contains("number"));
        assert!(message.contains("readonly"));
        assert!(!message.contains("timeoutlen"), "{command}: {message}");
    }
    assert!(query(&mut app, "set").contains("timeoutlen=17"));
    assert_eq!(query(&mut app, "setlocal tm?"), "  timeoutlen=17");
}

#[test]
fn missing_file_name_takes_priority_over_readonly_for_write() {
    let mut app = App::new(common::named_doc("draft", ""));
    run(&mut app, "setlocal ro");
    for command in ["w", "w!"] {
        run(&mut app, command);
        assert!(app.ui.message.contains("no associated file path"));
        assert_eq!(query(&mut app, "setlocal ro?"), "  readonly");
    }
}

#[test]
fn only_preserves_options_of_the_last_closed_window_for_a_future_first_visit() {
    let mut app = app("set-only-fallback");
    app.vsplit(None);
    app.vsplit(None);
    app.open_rel("b.txt");
    run(&mut app, "setlocal nu");
    app.focus_next();
    app.open_rel("b.txt");
    run(&mut app, "setlocal nonu");
    app.focus_next();
    app.only();
    app.open_rel("b.txt");
    assert_eq!(query(&mut app, "setlocal nu?"), "  number");
}
