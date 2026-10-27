mod common;

use std::fs::{self, File};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use v_ide_editor::disk::{DiskMark, Reaction};
use v_ide_editor::document::Document;

fn file(label: &str, text: &str) -> PathBuf {
    let path = common::scratch(&format!("edge-{label}")).join("note.md");
    fs::write(&path, text).unwrap();
    path
}

fn shift_mtime(path: &PathBuf) {
    let at = SystemTime::now() + Duration::from_secs(60);
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(at)
        .unwrap();
}

#[test]
fn same_length_rewrite_with_new_mtime_reloads() {
    let path = file("same-len", "abc");
    let mut doc = Document::open(&path).unwrap();
    fs::write(&path, "xyz").unwrap();
    shift_mtime(&path);
    assert_eq!(doc.check_disk(), Some(Reaction::Reload));
    assert_eq!(doc.editor.text(), "xyz");
}

#[test]
fn reload_waits_while_visual_selection_is_active() {
    let path = file("visual", "one\ntwo\nthree");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Gv");
    fs::write(&path, "x").unwrap();
    assert_eq!(doc.check_disk(), None);
    assert_eq!(doc.editor.text(), "one\ntwo\nthree");
    doc.editor.handle_keys("y");
    assert_eq!(doc.check_disk(), Some(Reaction::Reload));
    assert_eq!(doc.editor.text(), "x");
}

#[test]
fn reload_waits_until_insert_session_ends() {
    let path = file("insert", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("i");
    fs::write(&path, "theirs").unwrap();
    assert_eq!(doc.check_disk(), None);
    doc.editor.handle_keys("<Esc>");
    assert_eq!(doc.check_disk(), Some(Reaction::Reload));
    doc.editor.handle_keys("u");
    assert_eq!(doc.editor.text(), "one");
}

#[test]
fn conflict_is_marked_even_during_insert() {
    let path = file("insert-dirty", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Amine");
    fs::write(&path, "theirs").unwrap();
    assert_eq!(doc.check_disk(), Some(Reaction::Conflict));
    assert_eq!(doc.disk_mark(), Some(DiskMark::Changed));
}

#[test]
fn unreadable_change_conflicts_with_dirty_buffer() {
    let path = file("bad-dirty", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Amine<Esc>");
    fs::write(&path, [0xff, 0xfe, 0x00, 0x01]).unwrap();
    assert_eq!(doc.check_disk(), Some(Reaction::Conflict));
    assert_eq!(doc.disk_mark(), Some(DiskMark::Changed));
    assert_eq!(doc.editor.text(), "onemine");
}

#[test]
fn unreadable_change_is_ignored_by_clean_buffer() {
    let path = file("bad-clean", "one");
    let mut doc = Document::open(&path).unwrap();
    fs::write(&path, [0xff, 0xfe, 0x00, 0x01]).unwrap();
    assert_eq!(doc.check_disk(), None);
    assert_eq!(doc.disk_mark(), None);
    assert_eq!(doc.editor.text(), "one");
    assert_eq!(doc.check_disk(), None);
}

#[test]
fn reload_of_vanished_file_marks_it_deleted_once() {
    let path = file("reload-gone", "one");
    let mut doc = Document::open(&path).unwrap();
    fs::remove_file(&path).unwrap();
    assert!(doc.reload().is_err());
    assert_eq!(doc.disk_mark(), Some(DiskMark::Deleted));
    assert_eq!(doc.check_disk(), None);
    assert_eq!(doc.editor.text(), "one");
}

#[test]
fn reload_of_never_existing_file_leaves_no_mark() {
    let path = common::scratch("edge-never").join("new.md");
    let mut doc = Document::new("draft");
    doc.path = Some(path);
    assert!(doc.reload().is_err());
    assert_eq!(doc.disk_mark(), None);
    assert_eq!(doc.editor.text(), "draft");
}

#[test]
fn undo_back_to_clean_catches_up_with_disk() {
    let path = file("undo-clean", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Amine<Esc>");
    fs::write(&path, "theirs").unwrap();
    doc.check_disk();
    doc.editor.handle_keys("u");
    assert_eq!(doc.check_disk(), Some(Reaction::Reload));
    assert_eq!(doc.editor.text(), "theirs");
    assert_eq!(doc.disk_mark(), None);
}
