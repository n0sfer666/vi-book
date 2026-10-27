mod common;

use std::fs;
use std::path::PathBuf;
use v_ide_editor::disk::{DiskMark, Reaction};
use v_ide_editor::document::Document;

fn file(label: &str, text: &str) -> PathBuf {
    let path = common::scratch(&format!("disk-{label}")).join("note.md");
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn own_save_is_not_an_external_change() {
    let path = file("own-save", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Atwo<Esc>");
    doc.save().unwrap();
    assert_eq!(doc.check_disk(), None);
    assert_eq!(doc.disk_mark(), None);
}

#[test]
fn rewrite_with_same_content_is_ignored() {
    let path = file("same", "one");
    let mut doc = Document::open(&path).unwrap();
    fs::write(&path, "one").unwrap();
    assert_eq!(doc.check_disk(), None);
    assert_eq!(doc.editor.text(), "one");
}

#[test]
fn clean_buffer_takes_external_change() {
    let path = file("clean", "one");
    let mut doc = Document::open(&path).unwrap();
    fs::write(&path, "one\ntwo").unwrap();
    assert_eq!(doc.check_disk(), Some(Reaction::Reload));
    assert_eq!(doc.editor.text(), "one\ntwo");
    assert!(!doc.is_dirty());
    assert_eq!(doc.disk_mark(), None);
    assert_eq!(doc.check_disk(), None, "change is reported once");
}

#[test]
fn reload_is_one_undo_step() {
    let path = file("undo", "one");
    let mut doc = Document::open(&path).unwrap();
    fs::write(&path, "one\ntwo\nthree").unwrap();
    doc.check_disk();
    doc.editor.handle_keys("u");
    assert_eq!(doc.editor.text(), "one");
}

#[test]
fn cursor_survives_reload() {
    let path = file("cursor", "a\nb\nc");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("j");
    fs::write(&path, "a\nb\nc\nd").unwrap();
    doc.check_disk();
    assert_eq!(doc.editor.cursor().line, 1);
}

#[test]
fn cursor_clamps_when_disk_shrinks() {
    let path = file("shrink", "a\nb\nc");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("G");
    fs::write(&path, "only").unwrap();
    doc.check_disk();
    assert_eq!(doc.editor.cursor().line, 0);
}

#[test]
fn dirty_buffer_conflicts_and_keeps_text() {
    let path = file("dirty", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Amine<Esc>");
    fs::write(&path, "theirs").unwrap();
    assert_eq!(doc.check_disk(), Some(Reaction::Conflict));
    assert_eq!(doc.editor.text(), "onemine");
    assert!(doc.is_dirty());
    assert_eq!(doc.disk_mark(), Some(DiskMark::Changed));
}

#[test]
fn disk_back_to_saved_text_clears_changed_mark() {
    let path = file("back", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Amine<Esc>");
    fs::write(&path, "theirs").unwrap();
    doc.check_disk();
    fs::write(&path, "one").unwrap();
    assert_eq!(doc.check_disk(), None);
    assert_eq!(doc.disk_mark(), None);
    assert_eq!(doc.editor.text(), "onemine");
}

#[test]
fn deleted_file_is_marked_once_and_keeps_text() {
    let path = file("deleted", "one");
    let mut doc = Document::open(&path).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(doc.check_disk(), Some(Reaction::Vanished));
    assert_eq!(doc.check_disk(), None);
    assert_eq!(doc.disk_mark(), Some(DiskMark::Deleted));
    assert_eq!(doc.editor.text(), "one");
    assert!(!doc.is_dirty());
}

#[test]
fn reappeared_file_with_saved_text_clears_deleted_mark() {
    let path = file("back-same", "one");
    let mut doc = Document::open(&path).unwrap();
    fs::remove_file(&path).unwrap();
    doc.check_disk();
    fs::write(&path, "one").unwrap();
    assert_eq!(doc.check_disk(), None);
    assert_eq!(doc.disk_mark(), None);
}

#[test]
fn reappeared_file_with_new_text_reloads_clean_buffer() {
    let path = file("back-new", "one");
    let mut doc = Document::open(&path).unwrap();
    fs::remove_file(&path).unwrap();
    doc.check_disk();
    fs::write(&path, "two!").unwrap();
    assert_eq!(doc.check_disk(), Some(Reaction::Reload));
    assert_eq!(doc.editor.text(), "two!");
    assert_eq!(doc.disk_mark(), None);
}

#[test]
fn file_that_never_existed_is_not_marked_deleted() {
    let path = common::scratch("disk-never").join("new.md");
    let mut doc = Document::new("");
    doc.path = Some(path);
    assert_eq!(doc.check_disk(), None);
    assert_eq!(doc.disk_mark(), None);
}

#[test]
fn buffer_without_path_is_never_checked() {
    let mut doc = Document::new("text");
    assert_eq!(doc.check_disk(), None);
}

#[test]
fn save_recreates_deleted_file_and_clears_mark() {
    let path = file("recreate", "one");
    let mut doc = Document::open(&path).unwrap();
    fs::remove_file(&path).unwrap();
    doc.check_disk();
    doc.save().unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "one");
    assert_eq!(doc.disk_mark(), None);
    assert_eq!(doc.check_disk(), None);
}

#[test]
fn save_overwrites_conflict_and_clears_mark() {
    let path = file("overwrite", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Amine<Esc>");
    fs::write(&path, "theirs").unwrap();
    doc.check_disk();
    doc.save().unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "onemine");
    assert_eq!(doc.disk_mark(), None);
    assert_eq!(doc.check_disk(), None);
}

#[test]
fn reload_takes_disk_as_one_undo_step_and_clears_mark() {
    let path = file("take", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Amine<Esc>");
    fs::write(&path, "theirs").unwrap();
    doc.check_disk();
    doc.reload().unwrap();
    assert_eq!(doc.editor.text(), "theirs");
    assert!(!doc.is_dirty());
    assert_eq!(doc.disk_mark(), None);
    doc.editor.handle_keys("u");
    assert_eq!(doc.editor.text(), "onemine");
}

#[test]
fn reload_of_deleted_file_fails_and_keeps_buffer() {
    let path = file("take-gone", "one");
    let mut doc = Document::open(&path).unwrap();
    doc.editor.handle_keys("Amine<Esc>");
    fs::remove_file(&path).unwrap();
    doc.check_disk();
    assert!(doc.reload().is_err());
    assert_eq!(doc.editor.text(), "onemine");
    assert_eq!(doc.disk_mark(), Some(DiskMark::Deleted));
}
