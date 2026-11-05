mod common;

use common::feed::{self, app, press};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{fs, time::Instant};
use v_ide_keys::MapMode;
use v_ide_tui::{app::App, command_line};

const TRACE: &str = r#"
    local trace={}
    vibook.create_autocmd({'InsertEnter','InsertLeavePre','InsertLeave','CursorMoved','CursorMovedI','TextChanged','TextChangedI'}, {
      callback=function(e)trace[#trace+1]=e.event..':'..vibook.buf.get_cursor() end
    })
    vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','));trace={}end)
"#;

fn setup(label: &str) -> App {
    let mut app = app("seed");
    common::load_lua(&mut app, label, TRACE);
    app
}

fn escape(app: &mut App) {
    app.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
}

#[test]
fn insert_events_observe_cursor_phases_and_escape_does_not_repeat_text_changed() {
    let mut app = setup("frequent-insert");
    press(&mut app, "iab");
    escape(&mut app);
    command_line::run(&mut app, "Capture");
    assert_eq!(
        feed::text(&app),
        "InsertEnter:0,CursorMovedI:1,TextChangedI:1,CursorMovedI:2,TextChangedI:2,InsertLeavePre:2,InsertLeave:1,CursorMoved:1"
    );
}

#[test]
fn mapping_coalesces_typing_and_undo_in_the_final_mode() {
    for (label, keys, expected) in [
        (
            "frequent-mapped-insert",
            "iab<Esc>",
            "InsertEnter:0,InsertLeavePre:2,InsertLeave:1,CursorMoved:1,TextChanged:1",
        ),
        ("frequent-mapped-undo", "xu", "CursorMoved:0,TextChanged:0"),
    ] {
        let mut app = setup(label);
        feed::map(&mut app, MapMode::Normal, "Q", keys, false);
        press(&mut app, "Q");
        command_line::run(&mut app, "Capture");
        assert_eq!(feed::text(&app), expected, "{label}");
    }
}

#[test]
fn control_c_exits_insert_without_leave_events_and_reports_normal_text_change() {
    let mut app = setup("frequent-interrupt");
    press(&mut app, "ia");
    feed::ctrl(&mut app, 'c');
    command_line::run(&mut app, "Capture");
    assert_eq!(
        feed::text(&app),
        "InsertEnter:0,CursorMovedI:1,TextChangedI:1,CursorMoved:0,TextChanged:0"
    );
}

#[test]
fn visual_cursor_movement_uses_cursor_moved() {
    let mut app = setup("frequent-visual-move");
    press(&mut app, "vl");
    escape(&mut app);
    command_line::run(&mut app, "Capture");
    assert_eq!(feed::text(&app), "CursorMoved:1");
}

#[test]
fn cursor_moved_navigation_delivers_text_changed_to_the_actual_document_once() {
    let mut app = app("seed");
    common::load_lua(
        &mut app,
        "frequent-move-navigation",
        r#"
        local changes={}
        vibook.create_autocmd('CursorMoved',{once=true,callback=function()
          vibook.cmd('enew')
          vibook.buf.set_text('new text')
        end})
        vibook.create_autocmd('TextChanged',{callback=function(e)
          changes[#changes+1]=e.buf..':'..vibook.buf.get_text()
          vibook.buf.set_text('handler edit')
        end})
        vibook.command('Capture',function()vibook.buf.set_text(table.concat(changes,','))end)
        "#,
    );
    let original = app.workspace.current_id();
    press(&mut app, "l");
    app.timeout(Instant::now());
    app.timeout(Instant::now());

    assert_eq!(app.workspace.doc(original).unwrap().editor.text(), "seed");
    command_line::run(&mut app, "Capture");
    assert_eq!(feed::text(&app), "2:new text");
}

#[test]
fn insert_enter_navigation_redirects_typing_and_leave_pre_resolves_the_actual_buffer() {
    let mut app = app("seed");
    common::load_lua(
        &mut app,
        "frequent-insert-navigation",
        r#"
        vibook.cmd('enew | file second.md')
        vibook.buf.set_text('second')
        vibook.cmd('buffer 1')
        vibook.create_autocmd('InsertEnter',{once=true,callback=function()vibook.cmd('buffer 2')end})
        vibook.create_autocmd('InsertLeavePre',{once=true,callback=function()vibook.cmd('buffer 1')end})
        vibook.create_autocmd('InsertLeave',{callback=function(e)
          assert(e.buf==1)
          vibook.buf.set_text('leave in original')
        end})
        "#,
    );
    let original = app.workspace.current_id();

    press(&mut app, "iz");
    assert_eq!(feed::text(&app), "zsecond");
    let typed = app.workspace.current_id();
    escape(&mut app);

    assert_eq!(app.workspace.current_id(), original);
    assert_eq!(feed::text(&app), "leave in original");
    let doc = app.workspace.doc(typed).unwrap();
    assert_eq!(doc.editor.text(), "zsecond");
    assert!(doc.editor.idle_normal());
}

#[test]
fn reading_a_file_does_not_report_text_changed() {
    let mut app = setup("frequent-read-baseline");
    let root = common::scratch("frequent-read-files");
    fs::write(root.join("loaded.md"), "loaded").unwrap();
    app.workspace.set_cwd(root);
    app.open_rel("loaded.md");
    app.timeout(Instant::now());
    fs::write(app.workspace.absolute("loaded.md"), "external").unwrap();
    command_line::run(&mut app, "e!");
    app.timeout(Instant::now());
    command_line::run(&mut app, "Capture");
    assert_eq!(feed::text(&app), "");
}

#[test]
fn read_view_edit_delegation_emits_insert_enter_for_the_real_edit() {
    let mut app = setup("frequent-read-edit");
    command_line::run(&mut app, "toggle-view");

    press(&mut app, "a");
    escape(&mut app);
    command_line::run(&mut app, "Capture");

    assert_eq!(
        feed::text(&app),
        "InsertEnter:1,CursorMovedI:1,InsertLeavePre:1,InsertLeave:0,CursorMoved:0"
    );
}

#[test]
fn text_changed_i_callback_edits_share_the_insert_undo_transaction() {
    let mut app = app("seed");
    common::load_lua(
        &mut app,
        "frequent-insert-undo-callback",
        r#"
        vibook.create_autocmd('TextChangedI',{once=true,callback=function()
          vibook.buf.set_text('')
        end})
        "#,
    );

    press(&mut app, "ia");
    escape(&mut app);
    assert_eq!(feed::text(&app), "");
    press(&mut app, "u");
    assert_eq!(feed::text(&app), "seed");
    feed::ctrl(&mut app, 'r');
    assert_eq!(feed::text(&app), "");
}

#[test]
fn cli_markdown_read_sets_the_initial_view_when_reusing_the_blank_document() {
    let root = common::scratch("frequent-cli-read-view");
    fs::write(root.join("a.md"), "seed").unwrap();
    let mut app = App::new(v_ide_editor::document::Document::new(""));
    app.workspace.set_cwd(root);
    let original = app.workspace.current_id();

    app.begin_startup();
    assert!(app.open_initial_rel("a.md"));
    app.finish_startup();

    assert_eq!(app.workspace.current_id(), original);
    assert_eq!(app.ui.read.view, v_ide_tui::view::View::Read);
}

#[test]
fn buffer_cursor_changes_and_window_switches_emit_cursor_moved() {
    let mut app = app("seed");
    common::load_lua(
        &mut app,
        "frequent-switch-fixture",
        "vibook.cmd('enew | file second.txt');vibook.buf.set_text('second');vibook.cmd('buffer 1')",
    );
    command_line::run(&mut app, "buffer 2");
    press(&mut app, "ll");
    command_line::run(&mut app, "buffer 1");
    app.timeout(Instant::now());
    common::load_lua(&mut app, "frequent-switch", TRACE);

    for command in [":buffer 2", ":buffer 1"] {
        press(&mut app, command);
        app.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    }
    app.vsplit(None);
    app.timeout(Instant::now());
    app.focus_next();
    app.timeout(Instant::now());
    command_line::run(&mut app, "Capture");

    assert_eq!(
        feed::text(&app),
        "CursorMoved:2,CursorMoved:0,CursorMoved:0,CursorMoved:0"
    );
}

#[test]
fn insert_frequent_callbacks_distinguish_cursor_and_text_navigation() {
    for event in ["CursorMovedI", "TextChangedI"] {
        let label = format!("frequent-nav-{event}");
        let mut app = app("seed");
        let edit_other = if event == "TextChangedI" {
            "vibook.buf.set_text('other')"
        } else {
            ""
        };
        common::load_lua(
            &mut app,
            &label,
            &format!(
                r#"
                vibook.cmd('enew | file second.txt')
                vibook.buf.set_text('second')
                vibook.cmd('buffer 1')
                vibook.create_autocmd('{event}',{{once=true,callback=function()
                  vibook.cmd('buffer 2')
                  {edit_other}
                end}})
                "#,
            ),
        );
        let first = app.workspace.current_id();
        let second = app.workspace.pool()[0].id;

        press(&mut app, "iAB");
        escape(&mut app);

        let a = app.workspace.doc(first).unwrap();
        let b = app.workspace.doc(second).unwrap();
        if event == "CursorMovedI" {
            assert_eq!(app.workspace.current_id(), second);
            assert_eq!(a.editor.text(), "Aseed");
            assert_eq!(b.editor.text(), "Bsecond");
        } else {
            assert_eq!(app.workspace.current_id(), first);
            assert_eq!(a.editor.text(), "BAseed");
            assert_eq!(b.editor.text(), "other");
        }
        assert!(a.editor.idle_normal() && b.editor.idle_normal());
    }
}

#[test]
fn text_changed_navigation_defers_source_baseline_until_the_buffer_returns() {
    let root = common::scratch("frequent-text-return");
    fs::write(root.join("other.txt"), "second").unwrap();
    let mut app = app("seed");
    app.workspace.set_cwd(root);
    app.open_rel("other.txt");
    command_line::run(&mut app, "buffer 1");
    app.timeout(Instant::now());
    common::load_lua(
        &mut app,
        "frequent-text-return",
        r#"
        local trace={}
        vibook.create_autocmd('TextChanged',{callback=function(e)trace[#trace+1]=e.buf end})
        vibook.create_autocmd('TextChanged',{once=true,callback=function()vibook.cmd('buffer 2')end})
        vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
        "#,
    );
    press(&mut app, "x");
    assert_eq!(feed::text(&app), "second");
    for command in ["buffer 1", "buffer 2", "buffer 1"] {
        command_line::run(&mut app, command);
        app.timeout(Instant::now());
    }
    command_line::run(&mut app, "Capture");
    assert_eq!(feed::text(&app), "1,1");
}

#[test]
fn insert_enter_and_leave_callback_edits_are_undone_with_typing() {
    for phase in ["InsertEnter", "InsertLeavePre", "InsertLeave"] {
        let label = format!("frequent-undo-{phase}");
        let mut app = app("seed");
        common::load_lua(
            &mut app,
            &label,
            &format!(
                "vibook.create_autocmd('{phase}',{{once=true,callback=function()vibook.buf.set_text('handler')end}})"
            ),
        );

        press(&mut app, "iA");
        escape(&mut app);
        let edited = feed::text(&app);
        assert_eq!(
            edited,
            if phase == "InsertEnter" {
                "Ahandler"
            } else {
                "handler"
            }
        );
        press(&mut app, "u");
        assert_eq!(feed::text(&app), "seed", "{phase}");
        feed::ctrl(&mut app, 'r');
        assert_eq!(feed::text(&app), edited, "{phase}");
    }
}
