#![cfg(unix)]

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use v_ide_editor::{Error, document::Document};
use v_ide_tui::{app::App, command_line, requests};

fn mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

fn file(label: &str, permissions: u32) -> PathBuf {
    let path = common::scratch(label).join("note.txt");
    fs::write(&path, "one").unwrap();
    mode(&path, permissions);
    path
}

#[test]
fn opening_a_file_without_write_bits_sets_readonly_and_blocks_normal_save() {
    let path = file("auto-ro-open", 0o444);
    let mut doc = Document::open(&path).unwrap();
    assert!(doc.readonly());
    assert!(matches!(doc.save(), Err(Error::Readonly)));
    assert_eq!(fs::read_to_string(&path).unwrap(), "one");
}

#[test]
fn access_check_respects_owner_permissions_even_if_another_class_has_write_bits() {
    let path = file("auto-ro-owner", 0o464);
    assert!(!fs::metadata(&path).unwrap().permissions().readonly());
    let doc = Document::open(&path).unwrap();
    assert!(doc.readonly());
}

#[test]
fn explicit_reload_redetects_readonly_for_all_windows_without_changing_the_default() {
    let path = file("auto-ro-explicit", 0o444);
    let mut app = App::new(Document::open(&path).unwrap());
    app.vsplit(None);
    command_line::run(&mut app, "setglobal ro");
    mode(&path, 0o644);
    command_line::run(&mut app, "e!");
    assert!(!app.workspace.current().readonly());
    app.focus_next();
    assert!(!app.workspace.current().readonly());
    command_line::run(&mut app, "setglobal ro?");
    assert_eq!(app.ui.message, "  readonly");
    command_line::run(&mut app, "setlocal ro");
    command_line::run(&mut app, "e!");
    assert!(!app.workspace.current().readonly());
}

#[test]
fn mode_only_changes_redetect_readonly_and_request_redraw_without_text_changes() {
    let path = file("auto-ro-mode-only", 0o644);
    let mut app = App::new(Document::open(&path).unwrap());
    mode(&path, 0o444);
    assert!(requests::tick(&mut app));
    assert!(app.workspace.current().readonly());
    assert_eq!(app.workspace.current().editor.text(), "one");
    assert!(!requests::tick(&mut app));
    mode(&path, 0o644);
    assert!(requests::tick(&mut app));
    assert!(!app.workspace.current().readonly());
    app.workspace.current_mut().editor.handle_keys("u");
    assert_eq!(app.workspace.current().editor.text(), "one");
    assert!(!app.workspace.current().is_dirty());
}

#[test]
fn automatic_reload_preserves_manual_readonly_only_when_the_full_mode_is_unchanged() {
    let path = file("auto-ro-preserve", 0o644);
    let mut app = App::new(Document::open(&path).unwrap());
    command_line::run(&mut app, "setlocal ro");
    fs::write(&path, "longer").unwrap();
    requests::tick(&mut app);
    assert!(app.workspace.current().readonly());
    assert_eq!(app.workspace.current().editor.text(), "longer");
    mode(&path, 0o744);
    requests::tick(&mut app);
    assert!(!app.workspace.current().readonly());
    assert_eq!(app.workspace.current().editor.text(), "longer");
}

#[test]
fn dirty_buffers_keep_their_text_and_manual_option_until_an_explicit_reload() {
    let path = file("auto-ro-dirty", 0o644);
    let mut app = App::new(Document::open(&path).unwrap());
    app.workspace.current_mut().editor.handle_keys("Ax<Esc>");
    mode(&path, 0o444);
    requests::tick(&mut app);
    assert!(!app.workspace.current().readonly());
    assert_eq!(app.workspace.current().editor.text(), "onex");
    command_line::run(&mut app, "e!");
    assert!(app.workspace.current().readonly());
    assert_eq!(app.workspace.current().editor.text(), "one");
}

#[cfg(target_os = "macos")]
#[test]
fn acl_denial_is_detected_even_with_owner_write_permission() {
    let path = file("auto-ro-acl", 0o644);
    let status = std::process::Command::new("/bin/chmod")
        .args(["+a", "everyone deny write"])
        .arg(&path)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(!fs::metadata(&path).unwrap().permissions().readonly());
    let doc = Document::open(&path).unwrap();
    assert!(doc.readonly());
}
