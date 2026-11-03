use std::fs;
use v_ide_editor::{
    document::Document,
    execution::Context,
    options::{Access, Name, Value},
    workspace::Workspace,
};

#[test]
fn edit_and_write_complete_inline_and_keep_canonical_hidden_buffers() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ex-io-inline");
    fs::create_dir_all(&root).unwrap();
    let first = root.join("first.md");
    let second = root.join("second.md");
    fs::write(&first, "first").unwrap();
    fs::write(&second, "second").unwrap();
    let mut ws = Workspace::new(Document::new(""));
    {
        let cx = Context::new(&mut ws);
        cx.execute(&format!("edit {}", first.display())).unwrap();
        let id = cx.buffer(0).unwrap();
        cx.set_text("saved");
        cx.execute("write").unwrap();
        assert_eq!(fs::read_to_string(&first).unwrap(), "saved");
        cx.execute(&format!("e {}", second.display())).unwrap();
        assert_eq!(cx.text(), "second");
        cx.execute(&format!("edit {}", first.display())).unwrap();
        assert_eq!(cx.buffer(0), Some(id));
        assert_eq!(cx.text(), "saved");
        fs::write(&first, "external").unwrap();
        cx.set_text("modified");
        assert!(cx.execute("edit").is_err());
        cx.execute("edit!").unwrap();
        assert_eq!(cx.text(), "external");
    }
    assert!(!ws.current().is_dirty());
}

#[test]
fn writes_preserve_existing_readonly_and_disk_conflict_contracts() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ex-io-conflict");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("note.md");
    fs::write(&file, "initial").unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::new(&mut ws);
    cx.execute(&format!("edit {}", file.display())).unwrap();
    cx.set_text("own");
    fs::write(&file, "disk").unwrap();
    assert!(cx.execute("write").is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(), "disk");
    cx.execute("set readonly").unwrap();
    assert!(cx.execute("write").is_err());
    cx.execute("write!").unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "own");
    assert_eq!(cx.option(Name::Readonly, Access::Local), Value::Bool(false));
}

#[test]
fn writing_a_new_name_never_overwrites_an_existing_file_without_bang() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ex-no-overwrite");
    fs::create_dir_all(&root).unwrap();
    for (index, original) in ["", "seed"].into_iter().enumerate() {
        let file = root.join(format!("{index}.md"));
        fs::write(&file, original).unwrap();
        let mut ws = Workspace::new(Document::new(original));
        let cx = Context::new(&mut ws);
        cx.set_text("own");
        assert!(cx.execute(&format!("write {}", file.display())).is_err());
        assert_eq!(fs::read_to_string(&file).unwrap(), original);
        cx.execute("write!").unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "own");
    }
}

#[test]
fn forced_edit_selects_a_loaded_hidden_buffer_without_discarding_its_changes() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ex-hidden-force");
    fs::create_dir_all(&root).unwrap();
    let first = root.join("a.md");
    let second = root.join("b.md");
    fs::write(&first, "a").unwrap();
    fs::write(&second, "disk").unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::new(&mut ws);
    cx.execute(&format!("edit {}", first.display())).unwrap();
    let first_id = cx.buffer_number(cx.buffer(0).unwrap()).unwrap();
    cx.execute(&format!("edit {}", second.display())).unwrap();
    cx.set_text("dirty hidden");
    cx.execute(&format!("buffer {first_id}")).unwrap();
    cx.set_text("dirty first");
    cx.execute(&format!("edit! {}", second.display())).unwrap();
    assert_eq!(cx.text(), "dirty hidden");
    assert_eq!(fs::read_to_string(&second).unwrap(), "disk");
    cx.execute(&format!("buffer {first_id}")).unwrap();
    assert_eq!(cx.text(), "dirty first");
}

#[test]
fn forced_edit_of_a_missing_current_file_keeps_identity_and_resets_to_new_empty_text() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ex-missing-current");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("missing.md");
    let mut ws = Workspace::new(Document::new(""));
    {
        let cx = Context::new(&mut ws);
        cx.execute(&format!("edit {}", file.display())).unwrap();
        let id = cx.buffer(0).unwrap();
        cx.set_text("unsaved");
        cx.execute("edit!").unwrap();
        assert_eq!(cx.buffer(0), Some(id));
        assert_eq!(cx.text(), "");
    }
    assert!(!ws.current().is_dirty());
    assert!(!file.exists());
}
