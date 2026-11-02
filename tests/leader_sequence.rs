mod common;

use std::time::Instant;

use common::feed::{press, text};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use v_ide_keys::TIMEOUT;
use v_ide_tui::{app::App, command_line::run};

fn lua(label: &str, script: &str) -> App {
    common::lua_app(common::named_doc("one\ntwo\nthree", "a.md"), label, script)
}

#[test]
fn full_leader_sequence_opens_menu_without_running_its_prefix() {
    let mut app = lua("leader-sequence", "vibook.opt.leader = 'ab'");
    press(&mut app, "a");
    assert!(app.ui.menu.is_none());
    assert!(app.workspace.current().editor.idle_normal());
    press(&mut app, "b");
    assert!(app.ui.menu.is_some());
    assert_eq!(text(&app), "one\ntwo\nthree");
}

#[test]
fn leader_notation_and_expansion_work_in_lua_and_ex() {
    let mut app = lua(
        "leader-notation",
        "vibook.opt.leader = '<C-a>x'\nvibook.keymap('n', '<Leader>d', 'j')",
    );
    common::feed::ctrl(&mut app, 'a');
    press(&mut app, "xd");
    assert_eq!(app.workspace.current().editor.cursor().line, 1);
    run(&mut app, "nnoremap <Leader>u k");
    common::feed::ctrl(&mut app, 'a');
    press(&mut app, "xu");
    assert_eq!(app.workspace.current().editor.cursor().line, 0);
}

#[test]
fn longer_mapping_wins_and_leader_alone_opens_menu_after_timeout() {
    let mut app = lua(
        "leader-long-map",
        "vibook.opt.leader = 'ab'\nvibook.keymap('n', '<Leader>d', 'j')",
    );
    press(&mut app, "abd");
    assert!(app.ui.menu.is_none());
    assert_eq!(app.workspace.current().editor.cursor().line, 1);
    press(&mut app, "ab");
    assert!(app.ui.menu.is_none());
    app.timeout(Instant::now() + TIMEOUT);
    assert!(app.ui.menu.is_some());
}

#[test]
fn mismatch_and_timeout_return_the_prefix_to_ordinary_editing() {
    for suffix in [Some("c"), None] {
        let mut app = lua("leader-fallback", "vibook.opt.leader = 'ab'");
        press(&mut app, "a");
        match suffix {
            Some(suffix) => press(&mut app, suffix),
            None => app.timeout(Instant::now() + TIMEOUT),
        }
        assert!(app.ui.menu.is_none());
        app.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(
            text(&app),
            if suffix.is_some() {
                "ocne\ntwo\nthree"
            } else {
                "one\ntwo\nthree"
            }
        );
    }
}

#[test]
fn mapclear_does_not_remove_menu_and_noremap_rhs_can_open_it() {
    let mut app = lua(
        "leader-noremap",
        "vibook.opt.leader = 'ab'\nvibook.keymap('n', 'Q', '<Leader>')",
    );
    assert!(
        app.keymap
            .list(&[v_ide_keys::MapMode::Normal], &[])
            .iter()
            .all(|g| g.lhs != v_ide_keys::parse_keys("ab"))
    );
    press(&mut app, "Q");
    assert!(app.ui.menu.is_some());
    app.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    run(&mut app, "nmapclear");
    press(&mut app, "ab");
    assert!(app.ui.menu.is_some());
}

#[test]
fn empty_leader_reports_error_and_preserves_the_previous_sequence() {
    let mut app = lua("leader-before-empty", "vibook.opt.leader = 'ab'");
    let path = common::config_file("leader-empty", "vibook.opt.leader = ''");
    v_ide_tui::lua_bridge::load_config_from(&mut app, Some((path, None)));
    assert!(app.ui.message.contains("leader"));
    press(&mut app, "ab");
    assert!(app.ui.menu.is_some());
}
