mod common;

use common::feed::{THREE, app, ctrl, line, press, text};
use v_ide_engine::Mode;
use v_ide_keys::Key;
use v_ide_tui::command_line::run;

#[test]
fn ex_remap_and_noremap_follow_different_paths() {
    for (command, expected) in [("nmap", "one\nthree"), ("nnoremap", "two\nthree")] {
        let mut app = app(THREE);
        run(&mut app, "nnoremap dd jdd");
        run(&mut app, &format!("{command} Q dd"));
        press(&mut app, "Q");
        assert_eq!(text(&app), expected, "{command}");
    }
}

#[test]
fn map_targets_normal_visual_and_operator_pending() {
    for (keys, expected) in [
        ("Lx", "one\nwo\nthree"),
        ("vLd", "wo\nthree"),
        ("dL", "three"),
    ] {
        let mut app = app(THREE);
        run(&mut app, "noremap L j");
        press(&mut app, keys);
        assert_eq!(text(&app), expected, "{keys}");
    }
}

#[test]
fn bang_mapping_targets_insert_and_command_line() {
    let mut app = app("");
    run(&mut app, "noremap! qq XY");
    press(&mut app, "iqq");
    assert_eq!(text(&app), "XY");
    app.workspace
        .current_mut()
        .editor
        .handle_key(Key::new(v_ide_keys::Code::Esc, v_ide_keys::Mods::NONE));
    press(&mut app, ":qq");
    assert_eq!(app.ui.command.as_deref(), Some("XY"));
}

#[test]
fn cmd_rhs_preserves_insert_visual_and_command_line_modes() {
    for (mode, prefix) in [("i", "i"), ("x", "v"), ("c", ":draft")] {
        let mut app = app(THREE);
        run(&mut app, &format!("{mode}noremap <C-s> <Cmd>only<CR>"));
        press(&mut app, prefix);
        ctrl(&mut app, 's');
        assert_eq!(text(&app), THREE);
        match mode {
            "i" => assert_eq!(app.workspace.current().editor.mode(), Mode::Insert),
            "x" => assert_eq!(app.workspace.current().editor.mode(), Mode::Visual),
            _ => assert_eq!(app.ui.command.as_deref(), Some("draft")),
        }
    }
}

#[test]
fn colon_rhs_uses_the_command_line_and_pipe_is_literal() {
    let mut app = app("");
    run(&mut app, "nnoremap Q :only<CR>");
    press(&mut app, "Q");
    assert!(app.ui.command.is_none());
    assert!(app.ui.message.is_empty());
    run(&mut app, "nnoremap Z ia|b<Esc>");
    press(&mut app, "Z");
    assert_eq!(text(&app), "a|b");
}

#[test]
fn unmap_and_clear_remove_insert_escape_too() {
    for command in ["iunmap jj", "imapclear", "mapclear!"] {
        let mut app = app("");
        run(&mut app, command);
        press(&mut app, "ijj");
        assert_eq!(text(&app), "jj", "{command}");
        assert_eq!(app.workspace.current().editor.mode(), Mode::Insert);
    }
}

#[test]
fn unique_collision_preserves_all_original_modes() {
    let mut app = app(THREE);
    run(&mut app, "omap Q j");
    run(&mut app, "map <unique> Q dd");
    assert_eq!(app.ui.message, "E227: Mapping already exists for Q");
    press(&mut app, "Q");
    assert_eq!(text(&app), THREE);
    press(&mut app, "dQ");
    assert_eq!(text(&app), "three");
}

#[test]
fn unmap_errors_and_invalid_rhs_do_not_destroy_existing_maps() {
    let mut app = app(THREE);
    run(&mut app, "nunmap Q");
    assert_eq!(app.ui.message, "E31: No such mapping");
    run(&mut app, "nnoremap Q j");
    run(&mut app, "nnoremap Q <Cmd>w");
    assert_eq!(app.ui.message, "E1255: <Cmd> mapping must end with <CR>");
    press(&mut app, "Q");
    assert_eq!(line(&app), 1);
}

#[test]
fn mapping_error_flushes_expansion_but_keeps_typed_input() {
    let mut app = app(THREE);
    run(&mut app, "nnoremap Z :nunmap missing<CR>dd");
    for c in "Zj".chars() {
        app.typeahead.feed(Key::char(c));
    }
    app.timeout(std::time::Instant::now());
    assert_eq!(text(&app), THREE);
    assert_eq!(line(&app), 1);
}

#[test]
fn listing_does_not_flush_expansion() {
    let mut app = app(THREE);
    run(&mut app, "nnoremap Z :nmap<CR>j");
    press(&mut app, "Z");
    assert!(app.ui.message.contains(":nmap<CR>j"), "{}", app.ui.message);
    assert_eq!(line(&app), 1);
}

#[test]
fn nowait_and_nop_are_executable() {
    let mut app = app(THREE);
    run(&mut app, "nnoremap ab dd");
    run(&mut app, "nnoremap <nowait> a j");
    run(&mut app, "nnoremap Q <Nop>");
    press(&mut app, "aQ");
    assert_eq!(line(&app), 1);
    assert!(app.typeahead.deadline().is_none());
    assert_eq!(text(&app), THREE);
}
