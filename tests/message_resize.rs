mod common;

use common::feed::app;
use v_ide_tui::pager::Pager;

#[test]
fn narrower_window_reopens_paging_instead_of_hiding_wrapped_content() {
    let mut pager = Pager::new("abcdefghij\nklmnopqrst", 40, 6);
    assert!(!pager.more());
    pager.resize(3, 4);
    assert!(pager.more());
    assert_eq!(
        pager.visible().collect::<Vec<_>>(),
        vec!["abc", "def", "ghi"]
    );
}

#[test]
fn tiny_frames_and_resizing_keep_cursor_in_bounds_and_do_not_modify_the_document() {
    let mut app = app("document");
    app.ui.set_message("界界\nsecond\nthird");
    for (width, height) in [(0, 0), (1, 1), (2, 2), (40, 8)] {
        let frame = common::draw(&mut app, width, height);
        if let Some((col, row)) = frame.cursor {
            assert!(col < width && row < height);
        }
        assert_eq!(app.workspace.current().editor.text(), "document");
        assert!(app.ui.pager.is_some());
    }
}
