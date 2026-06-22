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

pub fn buffer_list_key(app: &mut App, event: KeyEvent) {
    let Some(list) = app.buffer_list.as_mut() else {
        return;
    };
    match event_map::to_key(event) {
        Some(Key::Esc) => app.buffer_list = None,
        Some(Key::Enter) => {
            let target = list.current().map(|e| e.idx);
            app.buffer_list = None;
            if let Some(idx) = target
                && idx > 0
            {
                app.switch_to_buffer(idx - 1);
            }
        }
        Some(Key::Ctrl('d')) => {
            if let Some(idx) = list.current().map(|e| e.idx) {
                app.close_buffer(idx);
                let entries = app.named_buffers();
                if entries.is_empty() {
                    app.buffer_list = None;
                } else if let Some(list) = app.buffer_list.as_mut() {
                    list.rebuild(entries);
                }
            }
        }
        Some(Key::Backspace) => list.pop(),
        Some(Key::Ctrl('n')) | Some(Key::Down) => list.move_down(),
        Some(Key::Ctrl('p')) | Some(Key::Up) => list.move_up(),
        Some(Key::Char(c)) => list.push(c),
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
