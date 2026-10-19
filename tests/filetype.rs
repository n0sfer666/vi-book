use std::path::{Path, PathBuf};

use v_ide_editor::document::Document;
use v_ide_editor::filetype::detect;

#[test]
fn markdown_extensions_are_detected() {
    assert_eq!(detect(Path::new("a.md")), Some("markdown"));
    assert_eq!(detect(Path::new("dir/b.markdown")), Some("markdown"));
    assert_eq!(detect(Path::new("C.MD")), Some("markdown"));
}

#[test]
fn unknown_or_missing_extension_has_no_filetype() {
    assert_eq!(detect(Path::new("main.rs")), None);
    assert_eq!(detect(Path::new("README")), None);
    assert_eq!(detect(Path::new("")), None);
}

#[test]
fn document_filetype_follows_path() {
    let mut doc = Document::new("x");
    assert_eq!(doc.filetype(), None);
    doc.path = Some(PathBuf::from("notes/n.md"));
    assert_eq!(doc.filetype(), Some("markdown"));
}
