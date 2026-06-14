use ratatui::Terminal;
use ratatui::backend::TestBackend;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use vi_book::Editor;
use vi_book::ui::app::App;
use vi_book::ui::picker::Picker;
use vi_book::ui::render;

fn background() -> Editor {
    let line = "X".repeat(70);
    let text = vec![line; 30].join("\n");
    Editor::new(&text)
}

fn render_app(app: &mut App) -> Vec<Vec<String>> {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| render::draw(frame, app)).unwrap();
    let buf = terminal.backend().buffer().clone();
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect()
        })
        .collect()
}

fn assert_modal_is_opaque(grid: &[Vec<String>]) {
    let mut top = None;
    let mut left = None;
    let mut bottom = None;
    let mut right = None;
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if cell == "\u{250c}" {
                top = Some(y);
                left = Some(x);
            }
            if cell == "\u{2518}" {
                bottom = Some(y);
                right = Some(x);
            }
        }
    }
    let (top, left, bottom, right) = (
        top.expect("no top-left corner"),
        left.expect("no top-left corner"),
        bottom.expect("no bottom-right corner"),
        right.expect("no bottom-right corner"),
    );
    for row in grid.iter().take(bottom).skip(top + 1) {
        for cell in row.iter().take(right).skip(left + 1) {
            assert_ne!(cell, "X", "background bleeds through the modal");
        }
    }
}

#[test]
fn help_modal_is_opaque() {
    let mut app = App::new(background(), "a.md".into());
    app.help = true;
    let grid = render_app(&mut app);
    assert_modal_is_opaque(&grid);
}

#[test]
fn picker_modal_is_opaque() {
    let mut app = App::new(background(), "a.md".into());
    app.picker = Some(Picker::new(vec!["one".into(), "two".into()]));
    let grid = render_app(&mut app);
    assert_modal_is_opaque(&grid);
}

#[test]
fn menu_modal_is_opaque() {
    let mut app = App::new(background(), "a.md".into());
    app.handle(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
    assert!(app.menu.is_some());
    let grid = render_app(&mut app);
    assert_modal_is_opaque(&grid);
}
