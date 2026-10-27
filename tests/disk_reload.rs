mod common;

use std::fs;
use std::path::PathBuf;

use v_ide_tui::app::App;
use v_ide_tui::{command_line, requests};

fn vault(label: &str) -> (App, PathBuf) {
    let root = common::scratch(&format!("reload-{label}"));
    fs::write(root.join("a.md"), "alpha").unwrap();
    fs::write(root.join("b.md"), "beta").unwrap();
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    app.open_rel("a.md");
    (app, root)
}

fn screen(app: &mut App) -> String {
    let grid = common::draw(app, 80, 24).grid;
    (0..grid.h)
        .map(|y| {
            (0..grid.w)
                .map(|x| grid.cell(x, y).symbol.as_str().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn conflict(label: &str) -> (App, PathBuf) {
    let (mut app, root) = vault(label);
    app.workspace.current_mut().editor.handle_keys("Amine<Esc>");
    fs::write(root.join("a.md"), "theirs").unwrap();
    requests::tick(&mut app);
    (app, root)
}

#[test]
fn quiet_tick_reports_nothing() {
    let (mut app, _root) = vault("quiet");
    app.ui.message.clear();
    assert!(!requests::tick(&mut app));
    assert_eq!(app.ui.message, "");
}

#[test]
fn tick_reloads_clean_current_buffer() {
    let (mut app, root) = vault("current");
    fs::write(root.join("a.md"), "alpha two").unwrap();
    assert!(requests::tick(&mut app), "reload redraws");
    assert_eq!(app.workspace.current().editor.text(), "alpha two");
    assert_eq!(app.ui.message, "\"a.md\" reloaded");
}

#[test]
fn tick_reloads_clean_pool_buffer() {
    let (mut app, root) = vault("pool");
    app.open_rel("b.md");
    fs::write(root.join("a.md"), "alpha two").unwrap();
    requests::tick(&mut app);
    let pooled = &app.workspace.pool()[0].doc;
    assert_eq!(pooled.name, "a.md");
    assert_eq!(pooled.editor.text(), "alpha two");
    assert_eq!(app.ui.message, "\"a.md\" reloaded");
}

#[test]
fn conflict_is_reported_and_marked_in_status() {
    let (mut app, _root) = conflict("status");
    assert_eq!(
        app.ui.message,
        "\"a.md\" changed on disk (:e! load, :w! overwrite)"
    );
    assert_eq!(app.workspace.current().editor.text(), "alphamine");
    assert!(screen(&mut app).contains("a.md [+] [changed]"));
}

#[test]
fn write_refuses_on_conflict() {
    let (mut app, root) = conflict("refuse");
    app.ui.message.clear();
    command_line::run(&mut app, "w");
    assert_eq!(fs::read_to_string(root.join("a.md")).unwrap(), "theirs");
    assert_eq!(
        app.ui.message,
        "\"a.md\" changed on disk (:e! load, :w! overwrite)"
    );
}

#[test]
fn write_quit_refuses_on_conflict() {
    let (mut app, root) = conflict("wq");
    command_line::run(&mut app, "wq");
    assert!(!app.quit);
    assert_eq!(fs::read_to_string(root.join("a.md")).unwrap(), "theirs");
}

#[test]
fn force_write_overwrites_and_clears_mark() {
    let (mut app, root) = conflict("force");
    command_line::run(&mut app, "w!");
    assert_eq!(fs::read_to_string(root.join("a.md")).unwrap(), "alphamine");
    assert!(!screen(&mut app).contains("[changed]"));
}

#[test]
fn edit_bang_takes_disk_and_undo_restores_mine() {
    let (mut app, _root) = conflict("take");
    command_line::run(&mut app, "e!");
    assert_eq!(app.workspace.current().editor.text(), "theirs");
    assert!(!screen(&mut app).contains("[changed]"));
    app.workspace.current_mut().editor.handle_keys("u");
    assert_eq!(app.workspace.current().editor.text(), "alphamine");
}

#[test]
fn edit_bang_without_conflict_reverts_to_disk() {
    let (mut app, _root) = vault("revert");
    app.workspace.current_mut().editor.handle_keys("Amine<Esc>");
    command_line::run(&mut app, "e!");
    assert_eq!(app.workspace.current().editor.text(), "alpha");
    assert!(!app.workspace.current().is_dirty());
}

#[test]
fn deleted_file_is_marked_in_status_and_buffer_list() {
    let (mut app, root) = vault("deleted");
    app.open_rel("b.md");
    fs::remove_file(root.join("a.md")).unwrap();
    fs::remove_file(root.join("b.md")).unwrap();
    requests::tick(&mut app);
    assert_eq!(app.ui.message, "\"b.md\" deleted on disk (+1 more)");
    assert!(screen(&mut app).contains("b.md [deleted]"));
    let labels: Vec<String> = app.named_buffers().iter().map(|e| e.label()).collect();
    assert!(labels.contains(&String::from("  a.md [deleted]")));
}

#[test]
fn write_recreates_deleted_file() {
    let (mut app, root) = vault("recreate");
    fs::remove_file(root.join("a.md")).unwrap();
    requests::tick(&mut app);
    command_line::run(&mut app, "w");
    assert_eq!(fs::read_to_string(root.join("a.md")).unwrap(), "alpha");
    assert!(!screen(&mut app).contains("[deleted]"));
}

#[test]
fn quit_is_allowed_for_clean_deleted_buffer() {
    let (mut app, root) = vault("quit");
    fs::remove_file(root.join("a.md")).unwrap();
    requests::tick(&mut app);
    command_line::run(&mut app, "q");
    assert!(app.quit);
}

#[test]
fn edit_bang_on_deleted_file_keeps_buffer() {
    let (mut app, root) = vault("take-gone");
    app.workspace.current_mut().editor.handle_keys("Amine<Esc>");
    fs::remove_file(root.join("a.md")).unwrap();
    requests::tick(&mut app);
    app.ui.message.clear();
    command_line::run(&mut app, "e!");
    assert_eq!(app.ui.message, "\"a.md\" deleted on disk");
    assert_eq!(app.workspace.current().editor.text(), "alphamine");
}
