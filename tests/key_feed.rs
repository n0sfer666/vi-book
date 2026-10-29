mod common;

use std::time::{Duration, Instant};

use common::feed::{THREE, app, ctrl, line, map, map_command, press, status, text};
use v_ide_keys::{MapMode, TIMEOUT};
use v_ide_tui::view::View;

#[test]
fn multi_key_mapping_waits_for_the_rest_and_shows_prefix() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "jj", "dd", false);
    press(&mut app, "j");
    assert_eq!((text(&app), line(&app)), (THREE.to_string(), 0));
    assert!(status(&mut app).trim_end().ends_with('j'));
    press(&mut app, "j");
    assert_eq!(text(&app), "two\nthree");
    assert!(!status(&mut app).trim_end().ends_with('j'));
}

#[test]
fn wide_prefix_is_shown_flush_right_with_padding() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "日日", "dd", false);
    press(&mut app, "日");
    assert!(status(&mut app).ends_with("日 "));
}

#[test]
fn timeout_resolves_prefix_as_plain_keys() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "jj", "dd", false);
    press(&mut app, "j");
    app.timeout(Instant::now() + TIMEOUT);
    assert_eq!((text(&app), line(&app)), (THREE.to_string(), 1));
}

#[test]
fn timeout_before_deadline_keeps_waiting() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "jj", "dd", false);
    press(&mut app, "j");
    let deadline = app.typeahead.deadline().expect("prefix sets deadline");
    app.timeout(deadline - Duration::from_millis(1));
    press(&mut app, "j");
    assert_eq!(text(&app), "two\nthree");
}

#[test]
fn mapping_beats_builtin_key() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "?", "dd", false);
    press(&mut app, "?");
    assert!(!app.ui.help);
    assert_eq!(text(&app), "two\nthree");
}

#[test]
fn noremap_rhs_reaches_builtin_key() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "Q", "?", false);
    press(&mut app, "Q");
    assert!(app.ui.help);
}

#[test]
fn count_before_mapping_applies_to_rhs() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "Q", "dd", false);
    press(&mut app, "2Q");
    assert_eq!(text(&app), "three");
}

#[test]
fn zero_inside_count_is_not_mapped() {
    let mut app = app(&"x\n".repeat(12));
    map(&mut app, MapMode::Normal, "0", "dd", false);
    press(&mut app, "10j");
    assert_eq!((text(&app), line(&app)), ("x\n".repeat(12), 10));
    press(&mut app, "0");
    assert_eq!(text(&app), "x\n".repeat(11));
}

#[test]
fn action_rhs_runs_command() {
    let mut app = app(THREE);
    map_command(&mut app, "Q", "help");
    press(&mut app, "Q");
    assert!(app.ui.help);
}

#[test]
fn command_line_uses_cmdline_mappings_only() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "a", "x", false);
    map(&mut app, MapMode::Cmdline, "b", "c", false);
    press(&mut app, ":ab");
    assert_eq!(app.ui.command.as_deref(), Some("ac"));
}

#[test]
fn overlay_keys_are_not_mapped() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "q", "dd", false);
    press(&mut app, "?q");
    assert!(!app.ui.help);
    assert_eq!(text(&app), THREE);
}

#[test]
fn window_command_argument_is_not_mapped() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "v", "dd", false);
    ctrl(&mut app, 'w');
    press(&mut app, "v");
    assert_eq!(app.pane_count(), 2);
    assert_eq!(text(&app), THREE);
}

#[test]
fn read_view_keys_are_not_mapped() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "j", "dd", false);
    app.ui.read.view = View::Read;
    press(&mut app, "j");
    assert_eq!(text(&app), THREE);
}
