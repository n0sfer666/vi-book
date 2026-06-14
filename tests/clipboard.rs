use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use vi_book::Editor;
use vi_book::ui::app::App;
use vi_book::ui::clipboard::{base64, osc52};
use vi_book::ui::view::View;

#[test]
fn base64_matches_known_vectors() {
    assert_eq!(base64(b""), "");
    assert_eq!(base64(b"a"), "YQ==");
    assert_eq!(base64(b"ab"), "YWI=");
    assert_eq!(base64(b"abc"), "YWJj");
    assert_eq!(base64(b"hello"), "aGVsbG8=");
}

#[test]
fn osc52_wraps_base64_payload() {
    assert_eq!(osc52("hi"), "\x1b]52;c;aGk=\x07");
}

#[test]
fn enter_on_a_code_block_copies_it() {
    let src = "# Title\n\n```rust\nfn main() {}\n```\n";
    let mut app = App::new(Editor::new(src), "a.md".into());
    app.view = View::Read;
    let rendered = vi_book::markdown::render::render(src, 80);
    let line = rendered
        .lines
        .iter()
        .position(|l| l.text().contains("fn main"))
        .expect("code line rendered");
    app.read_cursor = (line, 3);
    app.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.editor.yanked().as_deref(), Some("fn main() {}"));
    assert!(
        app.message.contains("code block"),
        "message should confirm the copy, got: {:?}",
        app.message
    );
}
