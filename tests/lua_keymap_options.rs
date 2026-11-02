mod common;

use common::feed::{press, text};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use v_ide_tui::app::App;

fn lua(label: &str, script: &str) -> App {
    common::lua_app(common::named_doc("one\ntwo\nthree", "a.md"), label, script)
}

#[test]
fn remap_option_expands_other_lua_mappings() {
    let mut app = lua(
        "lua-remap",
        "vibook.keymap('n', 'Q', 'Z', { remap = true })\n\
         vibook.keymap('n', 'Z', 'j')",
    );
    press(&mut app, "Q");
    assert_eq!(app.workspace.current().editor.cursor().line, 1);
}

#[test]
fn absent_and_false_remap_keep_builtin_keys() {
    for option in ["", ", { remap = false }"] {
        let mut app = lua(
            "lua-noremap",
            &format!("vibook.keymap('n', 'Q', 'j'{option})\nvibook.keymap('n', 'j', 'G')"),
        );
        press(&mut app, "Q");
        assert_eq!(app.workspace.current().editor.cursor().line, 1);
    }
}

#[test]
fn table_modes_keep_earlier_definitions_and_stop_at_invalid_mode() {
    let mut app = lua(
        "lua-table-modes",
        "vibook.keymap({ 'n', 'i', 's', 'c' }, 'Q', 'l')",
    );
    assert!(app.ui.message.contains("invalid mode"));
    press(&mut app, "Q");
    assert_eq!(app.workspace.current().editor.cursor().col, 1);
    press(&mut app, "iQ");
    assert_eq!(text(&app), "olne\ntwo\nthree");
    app.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    press(&mut app, ":Q");
    assert_eq!(app.ui.command.as_deref(), Some("Q"));
}

#[test]
fn colon_rhs_types_a_command_and_waits_for_enter() {
    let mut app = lua(
        "lua-colon-keys",
        "vibook.command('shout', function() vibook.buf.set_text('SHOUTED') end)\n\
         vibook.keymap('n', 'Q', ':shout')",
    );
    press(&mut app, "Q");
    assert_eq!(app.ui.command.as_deref(), Some("shout"));
    assert_eq!(text(&app), "one\ntwo\nthree");
    app.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(text(&app), "SHOUTED");
}

#[test]
fn nowait_avoids_waiting_for_a_longer_mapping() {
    let mut app = lua(
        "lua-nowait",
        "vibook.keymap('n', 'ab', 'G')\nvibook.keymap('n', 'a', 'j', { nowait = true })",
    );
    press(&mut app, "a");
    assert_eq!(app.workspace.current().editor.cursor().line, 1);
}

#[test]
fn unique_keeps_existing_mapping_and_silent_is_accepted() {
    let mut app = lua(
        "lua-unique",
        "vibook.keymap('n', 'Q', 'j', { silent = true })\n\
         vibook.keymap('n', 'Q', 'G', { unique = true })",
    );
    assert!(app.ui.message.contains("E227"));
    press(&mut app, "Q");
    assert_eq!(app.workspace.current().editor.cursor().line, 1);
}

#[test]
fn invalid_options_skip_the_mapping_without_aborting_config() {
    for options in [
        "true",
        "{ remap = 'true' }",
        "{ nowait = 1 }",
        "{ unique = {} }",
        "{ silent = 'yes' }",
        "{ buffer = true }",
    ] {
        let mut app = lua(
            "lua-invalid-options",
            &format!("vibook.keymap('n', 'Q', 'G', {options})\nvibook.keymap('n', 'Z', 'j')"),
        );
        assert!(app.ui.message.contains("keymap"));
        press(&mut app, "QZ");
        assert_eq!(app.workspace.current().editor.cursor().line, 1);
    }
}

#[test]
fn mode_list_is_snapshotted_at_definition_and_listings_stay_separate() {
    let mut app = lua(
        "lua-mode-snapshot",
        "local modes = { 'n', 'x', 'o' }\nvibook.keymap(modes, 'Q', 'l')\nmodes[1] = 'i'",
    );
    press(&mut app, "Q");
    assert_eq!(app.workspace.current().editor.cursor().col, 1);
    v_ide_tui::command_line::run(&mut app, "map Q");
    let lines: Vec<_> = app.ui.message.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.iter().any(|line| line.starts_with("n ")));
    assert!(lines.iter().any(|line| line.starts_with("x ")));
    assert!(lines.iter().any(|line| line.starts_with("o ")));
}
