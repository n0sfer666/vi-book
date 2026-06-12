use std::fs;
use std::path::PathBuf;

use vi_book::{Editor, Error, Mode, Position};

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vi-book-test-{}-{}", std::process::id(), name))
}

fn roundtrip(name: &str, content: &[u8]) {
    let path = temp_path(name);
    fs::write(&path, content).unwrap();
    let mut editor = Editor::open(&path).unwrap();
    editor.save().unwrap();
    let written = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(written, content, "{name}: saved bytes differ from original");
}

#[test]
fn open_save_is_byte_identical_without_trailing_newline() {
    roundtrip("noeol", b"hello\nworld");
}

#[test]
fn open_save_is_byte_identical_with_trailing_newline() {
    roundtrip("eol", b"hello\nworld\n");
}

#[test]
fn open_save_is_byte_identical_for_empty_file() {
    roundtrip("empty", b"");
}

#[test]
fn open_save_is_byte_identical_for_utf8() {
    roundtrip("utf8", "h\u{e9}llo \u{1f30d}\n".as_bytes());
}

#[test]
fn save_writes_exact_edited_buffer() {
    let path = temp_path("edited");
    fs::write(&path, b"abc\n").unwrap();
    let mut editor = Editor::open(&path).unwrap();
    editor.handle_keys("iX<Esc>");
    editor.save().unwrap();
    let written = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(written, b"Xabc\n");
}

#[test]
fn save_after_full_undo_restores_original_bytes() {
    let path = temp_path("undo");
    fs::write(&path, b"abc").unwrap();
    let mut editor = Editor::open(&path).unwrap();
    editor.handle_keys("xu");
    editor.save().unwrap();
    let written = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(written, b"abc");
}

#[test]
fn open_reads_content_and_starts_in_normal_mode() {
    let path = temp_path("read");
    fs::write(&path, b"a\nb").unwrap();
    let editor = Editor::open(&path).unwrap();
    let text = editor.text();
    fs::remove_file(&path).unwrap();
    assert_eq!(text, "a\nb");
    assert_eq!(editor.cursor(), Position::new(0, 0));
    assert_eq!(editor.mode(), Mode::Normal);
}

#[test]
fn save_without_path_fails() {
    let mut editor = Editor::new("x");
    assert!(matches!(editor.save(), Err(Error::NoPath)));
}

#[test]
fn open_missing_file_fails() {
    let result = Editor::open(temp_path("missing-file-does-not-exist"));
    assert!(matches!(result, Err(Error::Io(_))));
}
