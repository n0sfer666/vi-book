use crossterm::event::KeyEvent;

use crate::ui::app::App;
use crate::ui::panel::Activation;
use crate::ui::{event_map, link_follow};
use crate::vim::key::Key;

pub fn normal_builtin(app: &mut App, mapped: Option<Key>) -> bool {
    match mapped {
        Some(Key::Char(':')) => app.command = Some(String::new()),
        Some(Key::Ctrl('p')) => app.open_picker(),
        Some(Key::Ctrl('w')) if app.panel.is_some() => app.panel_focus = true,
        Some(Key::Ctrl('o')) => {
            if let Some(rel) = app.history.back() {
                app.load(&rel);
            }
        }
        Some(Key::Ctrl('i')) => {
            if let Some(rel) = app.history.forward() {
                app.load(&rel);
            }
        }
        Some(Key::Enter) => link_follow::follow_at_cursor(app),
        _ => return false,
    }
    true
}

pub fn panel_key(app: &mut App, key: Option<Key>) {
    let Some(panel) = app.panel.as_mut() else {
        return;
    };
    match key {
        Some(Key::Char('j')) => panel.move_down(),
        Some(Key::Char('k')) => panel.move_up(),
        Some(Key::Esc) | Some(Key::Ctrl('w')) => app.panel_focus = false,
        Some(Key::Enter) => match panel.activate() {
            Activation::Open(path) => app.open_rel(&path),
            Activation::Toggled | Activation::None => {}
        },
        _ => {}
    }
}

pub fn picker_key(app: &mut App, event: KeyEvent) {
    let Some(picker) = app.picker.as_mut() else {
        return;
    };
    match event_map::to_key(event) {
        Some(Key::Esc) => app.picker = None,
        Some(Key::Enter) => {
            if let Some(rel) = picker.current() {
                app.open_rel(&rel);
            } else {
                app.picker = None;
            }
        }
        Some(Key::Backspace) => picker.pop(),
        Some(Key::Ctrl('n')) => picker.move_down(),
        Some(Key::Ctrl('p')) => picker.move_up(),
        Some(Key::Char(c)) => picker.push(c),
        _ => {}
    }
}
