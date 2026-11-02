mod common;

use common::feed::{map, press, text};
use std::time::{Duration, Instant};
use v_ide_keys::MapMode;
use v_ide_tui::{app::App, command_line};

fn run(app: &mut App, command: &str) {
    command_line::run(app, command);
}

fn query(app: &mut App, command: &str) -> String {
    run(app, command);
    app.ui.message.clone()
}

#[test]
fn effective_global_and_local_scopes_follow_native_copy_rules() {
    let mut app = App::new(common::named_doc("one", "a.txt"));
    run(&mut app, "set nu ro tm=73");
    assert_eq!(
        query(&mut app, "setlocal nu? ro? tm?"),
        "  number\n  readonly\n  timeoutlen=73"
    );
    run(&mut app, "setglobal nonu noro");
    assert_eq!(query(&mut app, "set nu? ro?"), "  number\n  readonly");
    assert_eq!(query(&mut app, "setglobal nu? ro?"), "nonumber\nnoreadonly");
    run(&mut app, "setlocal nu< ro<");
    run(&mut app, "setglobal nu ro");
    assert_eq!(query(&mut app, "set nu? ro?"), "nonumber\nnoreadonly");
    run(&mut app, "set invnu invro");
    assert_eq!(query(&mut app, "setglobal nu? ro?"), "  number\n  readonly");
    run(&mut app, "setlocal tm=29");
    assert_eq!(query(&mut app, "setglobal tm?"), "  timeoutlen=29");
}

#[test]
fn number_is_window_local_and_readonly_is_shared_by_splits() {
    let mut app = App::new(common::named_doc("one", "a.txt"));
    run(&mut app, "setlocal nu ro");
    app.vsplit(None);
    assert_eq!(query(&mut app, "set nu? ro?"), "  number\n  readonly");
    run(&mut app, "setlocal nonu noro");
    app.focus_next();
    assert_eq!(query(&mut app, "set nu? ro?"), "  number\nnoreadonly");
    app.focus_next();
    assert_eq!(query(&mut app, "set nu?"), "nonumber");
}

#[test]
fn new_buffers_get_global_number_but_not_global_readonly_and_keep_window_history() {
    let root = common::scratch("set-window-history");
    std::fs::write(root.join("a.txt"), "alpha").unwrap();
    std::fs::write(root.join("b.txt"), "beta").unwrap();
    let mut app = App::new(common::named_doc("", ""));
    app.workspace.set_cwd(root);
    app.open_rel("a.txt");
    run(&mut app, "setlocal nu ro");
    run(&mut app, "setglobal nonu ro");
    app.open_rel("b.txt");
    assert_eq!(query(&mut app, "set nu? ro?"), "nonumber\nnoreadonly");
    run(&mut app, "setglobal nu");
    assert_eq!(query(&mut app, "set nu?"), "nonumber");
    app.open_rel("a.txt");
    assert_eq!(query(&mut app, "set nu? ro?"), "  number\n  readonly");
    app.vsplit(Some("b.txt"));
    assert_eq!(query(&mut app, "set nu? ro?"), "nonumber\nnoreadonly");
    app.focus_next();
    app.open_rel("b.txt");
    assert_eq!(query(&mut app, "set nu?"), "nonumber");
}

#[test]
fn errors_keep_the_valid_prefix_and_reset_respects_scope() {
    let mut app = App::new(common::named_doc("one", "a.txt"));
    run(&mut app, "set nu tm=7 unknown ro");
    assert_eq!(app.ui.message, "E518: Unknown option: unknown ");
    assert_eq!(
        query(&mut app, "set nu? tm? ro?"),
        "  number\n  timeoutlen=7\nnoreadonly"
    );
    run(&mut app, "setlocal all&");
    assert_eq!(
        query(&mut app, "set nu? tm?"),
        "nonumber\n  timeoutlen=1000"
    );
    assert_eq!(query(&mut app, "setglobal nu?"), "  number");
    run(&mut app, "set all&");
    assert_eq!(query(&mut app, "setglobal nu?"), "nonumber");
}

#[test]
fn timeout_changes_apply_to_pending_and_subsequent_mapping_prefixes() {
    let mut app = App::new(common::named_doc("abc", "a.txt"));
    map(&mut app, MapMode::Normal, "Q", "x", false);
    map(&mut app, MapMode::Normal, "QQ", "xx", false);
    press(&mut app, "Q");
    assert_eq!(text(&app), "abc");
    run(&mut app, "set tm=0");
    app.timeout(Instant::now());
    assert_eq!(text(&app), "bc");
    press(&mut app, "Q");
    assert_eq!(text(&app), "c");
    run(&mut app, "set tm=30");
    press(&mut app, "Q");
    assert_eq!(text(&app), "c");
    app.timeout(Instant::now() + Duration::from_millis(40));
    assert_eq!(text(&app), "");
}

#[test]
fn set_error_discards_the_remaining_mapped_rhs() {
    let mut app = App::new(common::named_doc("abc", "a.txt"));
    run(&mut app, "nnoremap Q <Cmd>set tm=-1<CR>x");
    press(&mut app, "Q");
    assert_eq!(text(&app), "abc");
    assert!(app.ui.message.starts_with("E487"));
}
