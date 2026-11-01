mod common;

use common::feed::{THREE, app, line, press, text};
use v_ide_tui::command_line::run;

#[test]
fn mixed_ex_rhs_runs_commands_and_keys_in_order() {
    let mut app = app(THREE);
    run(&mut app, "nnoremap Q j<Cmd>vsplit<CR>k<Cmd>only<CR>x");
    press(&mut app, "Q");
    assert_eq!(app.pane_count(), 1);
    assert_eq!(text(&app), "ne\ntwo\nthree");
    assert_eq!(line(&app), 0);
}

#[test]
fn failed_cmd_discards_remaining_commands_and_keys() {
    let mut app = app(THREE);
    run(&mut app, "nnoremap Q j<Cmd>missing<CR>dd<Cmd>q!<CR>");
    press(&mut app, "Q");
    assert_eq!(text(&app), THREE);
    assert_eq!(line(&app), 1);
    assert!(!app.quit);
    assert!(app.ui.message.contains("not an editor command"));
}

#[test]
fn lua_mixed_cmd_rhs_preserves_command_order() {
    let mut app = common::lua_app(
        common::named_doc("a\nb\nc", "a.md"),
        "km-mixed-cmd",
        "vibook.keymap('n', 'Q', 'j<Cmd>vsplit<CR>k<Cmd>only<CR>x')",
    );
    press(&mut app, "Q");
    assert_eq!(app.pane_count(), 1);
    assert_eq!(text(&app), "\nb\nc");
}

#[test]
fn lua_reports_unterminated_cmd_and_skips_mapping() {
    let mut app = common::lua_app(
        common::named_doc(THREE, "a.md"),
        "km-invalid-cmd",
        "vibook.keymap('n', 'Q', 'j<Cmd>only')",
    );
    assert_eq!(app.ui.message, "E1255: <Cmd> mapping must end with <CR>");
    press(&mut app, "Q");
    assert_eq!(line(&app), 0);
    assert_eq!(text(&app), THREE);
}
