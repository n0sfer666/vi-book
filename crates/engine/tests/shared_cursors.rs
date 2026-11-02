use v_ide_engine::Editor;
use v_ide_text::Position;

#[test]
fn saved_view_restores_desired_column_as_well_as_cursor() {
    let mut editor = Editor::new("abcdef\nxy\nabcdefghijkl");
    editor.handle_keys("$");
    let view = editor.cursor_state();
    editor.handle_keys("0j");
    editor.restore_cursor_state(&view);
    editor.handle_keys("jj");
    assert_eq!(editor.cursor(), Position::new(2, 11));
}

#[test]
fn inactive_cursor_adjustment_matches_native_semantic_line_operations() {
    for (line, col, keys, expected) in [
        (0, 0, "Onew<Esc>", Position::new(2, 2)),
        (1, 0, "iXX<Esc>", Position::new(1, 2)),
        (1, 0, "xx", Position::new(1, 1)),
        (1, 0, "dd", Position::new(0, 0)),
        (0, 0, "dd", Position::new(0, 2)),
        (1, 0, "i<CR><Esc>", Position::new(1, 0)),
        (1, 0, "O<Esc>", Position::new(2, 2)),
        (1, 0, "i<CR><Esc>u", Position::new(0, 0)),
        (1, 0, "O<Esc>u", Position::new(1, 2)),
        (1, 0, "O<Esc>u<C-r>", Position::new(2, 2)),
        (0, 0, "3ddu", Position::new(0, 0)),
    ] {
        let mut editor = Editor::new("alpha\nbeta\ngamma");
        editor.set_cursor(Position::new(1, 2));
        let view = editor.cursor_state();
        editor.set_cursor(Position::new(line, col));
        editor.handle_keys(keys);
        editor.restore_cursor_state(&view);
        assert_eq!(editor.cursor(), expected, "{keys}");
    }
}

#[test]
fn replace_all_preserves_view_coordinates_then_clamps_to_current_text() {
    let mut editor = Editor::new("alpha\nbeta\ngamma");
    editor.set_cursor(Position::new(2, 3));
    let view = editor.cursor_state();
    editor.replace_all("x");
    editor.restore_cursor_state(&view);
    assert_eq!(editor.cursor(), Position::new(0, 0));
}
