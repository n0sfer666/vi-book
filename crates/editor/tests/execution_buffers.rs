use v_ide_editor::{document::Document, execution::Context, workspace::Workspace};

#[test]
fn buffer_zero_resolves_to_current_while_explicit_numbers_keep_their_identity() {
    let mut ws = Workspace::new(Document::new("first"));
    let first = ws.current_id();
    let second = ws.add(Document::new("second"));
    ws.select(second);
    let cx = Context::new(&mut ws);
    assert_eq!(cx.buffer(0), Some(second));
    assert_eq!(cx.buffer(1), Some(first));
    assert_eq!(cx.buffer(2), Some(second));
    assert_eq!(cx.buffer_number(first), Some(1));
    assert_eq!(cx.buffer_number(second), Some(2));
    assert_eq!(cx.buffer(3), None);
    assert_eq!(cx.buffer(u64::MAX), None);
}

#[test]
fn removed_buffer_numbers_are_invalid_and_are_never_reused() {
    let mut ws = Workspace::new(Document::new("first"));
    let removed = ws.add(Document::new("second"));
    ws.remove(0).unwrap();
    let third = ws.add(Document::new("third"));
    let cx = Context::new(&mut ws);
    assert_eq!(cx.buffer(2), None);
    assert_eq!(cx.buffer_number(removed), None);
    assert_eq!(cx.buffer(3), Some(third));
    assert_eq!(cx.buffer_number(third), Some(3));
}

#[test]
fn shared_windows_keep_one_buffer_number_and_leave_no_workspace_borrow_held() {
    let mut ws = Workspace::new(Document::new("shared"));
    let shared = ws.current_id();
    ws.split(ws.current_buffer()).unwrap();
    let observer = |cx: &Context<'_>, _: &v_ide_editor::execution::OptionSet| {
        let id = cx.buffer(0).unwrap();
        assert_eq!(id, shared);
        assert_eq!(cx.buffer_number(id), Some(1));
        cx.execute("set nonumber").unwrap();
        Ok(())
    };
    let cx = Context::with_observer(&mut ws, &observer);
    assert_eq!(cx.buffer(0), Some(shared));
    assert_eq!(cx.buffer(2), None);
    cx.execute("set number").unwrap();
}
