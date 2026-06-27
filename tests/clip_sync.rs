use vi_book::ui::clip_sync::freshest;
use vi_book::Editor;

#[test]
fn freshest_prefers_externally_changed_system_clipboard() {
    assert_eq!(freshest("same", "same"), None);
    assert_eq!(freshest("external", "ours"), Some("external"));
    assert_eq!(freshest("", "ours"), None);
    assert_eq!(freshest("external", ""), Some("external"));
}

#[test]
fn only_yank_bumps_the_clipboard_version() {
    let mut ed = Editor::new("hello world");
    let v0 = ed.clip_yank_version();
    ed.handle_keys("yw");
    let v1 = ed.clip_yank_version();
    assert!(v1 > v0, "yank should bump version");
    ed.handle_keys("x");
    assert_eq!(ed.clip_yank_version(), v1, "delete must not bump version");
    ed.handle_keys("dd");
    assert_eq!(ed.clip_yank_version(), v1, "dd must not bump version");
}

#[test]
fn injected_unnamed_register_is_what_paste_uses() {
    let mut ed = Editor::new("abc");
    let v = ed.clip_yank_version();
    ed.set_unnamed("XYZ".to_string(), false);
    assert_eq!(ed.clip_yank_version(), v, "injection must not bump version");
    ed.handle_keys("p");
    assert_eq!(ed.text(), "aXYZbc");
}
