use v_ide_editor::{
    autocmd::EventName, document::Document, execution::Context, workspace::Workspace,
};

#[path = "fixtures/autocmd_names.rs"]
mod native;

#[test]
fn every_native_event_distinguishes_file_matches_from_literal_matches() {
    let mut ws = Workspace::new(Document::new(""));
    let root = std::env::current_dir().unwrap();
    ws.set_cwd(root.clone());
    let cx = Context::new(&mut ws);
    for &(event, file) in native::CASES {
        let event = EventName::parse(event).unwrap();
        assert_eq!(event.is_file_pattern(), file);
        let names = cx
            .autocmd_names(event, cx.buffer(0).unwrap(), Some("relative-name"))
            .unwrap();
        assert_eq!(names.file, "relative-name");
        let expected = if file {
            root.join("relative-name").to_string_lossy().into_owned()
        } else {
            "relative-name".into()
        };
        assert_eq!(names.full, expected, "{}", event.as_str());
    }
}

#[path = "fixtures/autocmd_paths.rs"]
mod paths;

#[test]
fn file_matches_follow_native_fullname_formatting_for_relative_tilde_and_url_names() {
    let mut ws = Workspace::new(Document::new(""));
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("exec-names");
    std::fs::create_dir_all(root.join("target")).unwrap();
    ws.set_cwd(root.clone());
    let cx = Context::new(&mut ws);
    for &(path, expected) in paths::PATHS {
        let names = cx
            .autocmd_names(
                EventName::parse("BufEnter").unwrap(),
                cx.buffer(0).unwrap(),
                Some(path),
            )
            .unwrap();
        let expected = expected
            .replace("$CWD", &root.to_string_lossy())
            .replace("$PARENT", &root.parent().unwrap().to_string_lossy());
        assert_eq!(names.full, expected, "{path}");
        assert_eq!(names.file, path);
    }
}
