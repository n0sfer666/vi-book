use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use v_ide_keys::{MapMode, Mapping, Rhs, parse_keys};
use v_ide_tui::app::App;
use v_ide_tui::map_action::MapAction;

pub const THREE: &str = "one\ntwo\nthree";

pub fn app(text: &str) -> App {
    App::new(super::named_doc(text, "a.md"))
}

pub fn map(app: &mut App, mode: MapMode, lhs: &str, rhs: &str, remap: bool) {
    insert(app, mode, lhs, Rhs::Keys(parse_keys(rhs)), remap);
}

pub fn map_command(app: &mut App, lhs: &str, command: &str) {
    let rhs = Rhs::Action(MapAction::Command(String::from(command)));
    insert(app, MapMode::Normal, lhs, rhs, false);
}

fn insert(app: &mut App, mode: MapMode, lhs: &str, rhs: Rhs<MapAction>, remap: bool) {
    let mapping = Mapping { rhs, remap };
    assert!(app.keymap.insert(mode, &parse_keys(lhs), mapping).is_ok());
}

pub fn press(app: &mut App, chars: &str) {
    for c in chars.chars() {
        app.handle(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
}

pub fn ctrl(app: &mut App, c: char) {
    app.handle(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
}

pub fn text(app: &App) -> String {
    app.workspace.current().editor.text()
}

pub fn line(app: &App) -> usize {
    app.workspace.current().editor.cursor().line
}

pub fn status(app: &mut App) -> String {
    let grid = super::draw(app, 40, 8).grid;
    (0..8)
        .map(|y| grid.row_text(y))
        .find(|row| row.contains("NORMAL"))
        .unwrap_or_default()
}
