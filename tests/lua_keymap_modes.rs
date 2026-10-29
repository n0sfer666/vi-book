mod common;

use std::time::Instant;

use common::feed::{ctrl, press, text};
use v_ide_engine::mode::Mode;
use v_ide_keys::TIMEOUT;
use v_ide_tui::app::App;

const THREE: &str = "one\ntwo\nthree";
const SHOUT: &str = "vibook.command('shout', function() vibook.buf.set_text('SHOUTED') end)\n";

fn lua(text: &str, label: &str, script: &str) -> App {
    common::lua_app(common::named_doc(text, "a.md"), label, script)
}

fn mode(app: &App) -> Mode {
    app.workspace.current().editor.mode()
}

#[test]
fn insert_keymap_feeds_keys() {
    let mut app = lua("", "km-insert", "vibook.keymap('i', 'qq', 'XY')");
    press(&mut app, "iaqq");
    assert_eq!(text(&app), "aXY");
    assert_eq!(mode(&app), Mode::Insert);
}

#[test]
fn visual_keymap_with_v_mode() {
    let mut app = lua("abc\ndef", "km-v", "vibook.keymap('v', 'Z', 'd')");
    press(&mut app, "vlZ");
    assert_eq!(text(&app), "c\ndef");
}

#[test]
fn visual_keymap_with_x_mode() {
    let mut app = lua("abc\ndef", "km-x", "vibook.keymap('x', 'Z', 'd')");
    press(&mut app, "vlZ");
    assert_eq!(text(&app), "c\ndef");
}

#[test]
fn operator_pending_keymap_is_a_motion() {
    let mut app = lua("one two\nx", "km-o", "vibook.keymap('o', 'L', '$')");
    press(&mut app, "wdL");
    assert_eq!(text(&app), "one \nx");
}

#[test]
fn cmdline_keymap_types_into_command_line() {
    let mut app = lua(THREE, "km-c", "vibook.keymap('c', '<C-l>', 'abc')");
    press(&mut app, ":");
    ctrl(&mut app, 'l');
    assert_eq!(app.ui.command.as_deref(), Some("abc"));
}

#[test]
fn multi_key_normal_lhs() {
    let mut app = lua(THREE, "km-multi", "vibook.keymap('n', 'gh', 'dd')");
    press(&mut app, "gh");
    assert_eq!(text(&app), "two\nthree");
}

#[test]
fn leader_in_lhs_uses_leader_set_later_in_config() {
    let mut app = lua(
        THREE,
        "km-leader",
        "vibook.keymap('n', '<leader>D', 'dd')\nvibook.opt.leader = ','",
    );
    press(&mut app, ",D");
    assert_eq!(text(&app), "two\nthree");
}

#[test]
fn lone_leader_waits_for_timeout_then_opens_menu() {
    let mut app = lua(
        THREE,
        "km-leader-alone",
        "vibook.keymap('n', '<Leader>d', 'dd')",
    );
    press(&mut app, " ");
    assert!(app.ui.menu.is_none());
    app.timeout(Instant::now() + TIMEOUT);
    assert!(app.ui.menu.is_some());
}

#[test]
fn command_rhs_in_insert_keeps_insert_mode() {
    let script = format!("{SHOUT}vibook.keymap('i', '<C-s>', ':shout')");
    let mut app = lua("hi", "km-cmd-i", &script);
    press(&mut app, "i");
    ctrl(&mut app, 's');
    assert_eq!(text(&app), "SHOUTED");
    assert_eq!(mode(&app), Mode::Insert);
}

#[test]
fn command_rhs_in_cmdline_keeps_command_line_open() {
    let script = format!("{SHOUT}vibook.keymap('c', '<C-s>', ':shout')");
    let mut app = lua("hi", "km-cmd-c", &script);
    press(&mut app, ":ab");
    ctrl(&mut app, 's');
    assert_eq!(text(&app), "SHOUTED");
    assert_eq!(app.ui.command.as_deref(), Some("ab"));
}

#[test]
fn unknown_mode_is_reported_and_skipped() {
    let mut app = lua(THREE, "km-bad-mode", "vibook.keymap('s', 'Q', 'dd')");
    assert!(app.ui.message.contains("keymap"), "{}", app.ui.message);
    press(&mut app, "Q");
    assert_eq!(text(&app), THREE);
}

#[test]
fn empty_lhs_is_reported() {
    let app = lua(THREE, "km-empty-lhs", "vibook.keymap('n', '', 'dd')");
    assert!(app.ui.message.contains("keymap"), "{}", app.ui.message);
}

#[test]
fn leader_in_rhs_is_expanded() {
    let mut app = lua(
        THREE,
        "km-leader-rhs",
        "vibook.keymap('n', 'Q', '<Leader>')",
    );
    press(&mut app, "Q");
    assert!(app.ui.menu.is_some());
    assert_eq!(text(&app), THREE);
}

#[test]
fn table_mode_is_reported_and_skipped() {
    let mut app = lua(THREE, "km-table-mode", "vibook.keymap({ 'n' }, 'Q', 'dd')");
    assert!(app.ui.message.contains("keymap"), "{}", app.ui.message);
    press(&mut app, "Q");
    assert_eq!(text(&app), THREE);
}

#[test]
fn function_rhs_is_reported() {
    let app = lua(
        THREE,
        "km-fn-rhs",
        "vibook.keymap('n', 'Q', function() end)",
    );
    assert!(app.ui.message.contains("keymap"), "{}", app.ui.message);
}

#[test]
fn config_error_wins_over_keymap_error() {
    let app = lua(
        THREE,
        "km-error-order",
        "vibook.keymap('s', 'Q', 'dd')\nerror('boom')",
    );
    assert!(app.ui.message.contains("boom"), "{}", app.ui.message);
}
