use crossterm::event::KeyEvent;

use crate::ui::app::App;
use crate::ui::panel::{Activation, Panel};
use crate::ui::{event_map, link_follow};
use crate::vim::key::Key;

pub fn global_nav(app: &mut App, mapped: Option<Key>) -> bool {
    match mapped {
        Some(Key::Char(':')) => app.command = Some(String::new()),
        Some(Key::Ctrl('p')) => app.open_picker(),
        Some(Key::Ctrl('o')) => {
            if let Some(rel) = app.history.back() {
                app.open_rel(&rel);
            }
        }
        Some(Key::Ctrl('i')) => {
            if let Some(rel) = app.history.forward() {
                app.open_rel(&rel);
            }
        }
        _ => return false,
    }
    true
}

pub fn normal_builtin(app: &mut App, mapped: Option<Key>) -> bool {
    if global_nav(app, mapped) {
        return true;
    }
    match mapped {
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
        Some(Key::Char('j') | Key::Char('w') | Key::Char('W')) | Some(Key::Down) => {
            panel.move_down();
        }
        Some(Key::Char('k') | Key::Char('b') | Key::Char('B')) | Some(Key::Up) => panel.move_up(),
        Some(Key::Esc) | Some(Key::Ctrl('w')) => app.panel_focus = false,
        Some(Key::Enter) => {
            let close_after = matches!(panel, Panel::Backlinks { .. });
            match panel.activate() {
                Activation::Open(path) => {
                    app.open_rel(&path);
                    if close_after {
                        app.panel = None;
                        app.panel_focus = false;
                    }
                }
                Activation::Toggled | Activation::None => {}
            }
        }
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
        Some(Key::Ctrl('n')) | Some(Key::Down) => picker.move_down(),
        Some(Key::Ctrl('p')) | Some(Key::Up) => picker.move_up(),
        Some(Key::Char(c)) => picker.push(c),
        _ => {}
    }
}
