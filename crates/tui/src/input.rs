use crossterm::event::KeyEvent;

use crate::app::App;
use crate::panel::Activation;
use crate::{event_map, link_follow, requests};
use v_ide_keys::key::Key;

pub fn global_nav(app: &mut App, mapped: Option<Key>) -> bool {
    match mapped {
        Some(Key::Char(':')) => app.ui.command = Some(String::new()),
        #[cfg(feature = "vault")]
        Some(Key::Ctrl('p')) => app.open_picker(),
        #[cfg(feature = "notebook")]
        Some(Key::Ctrl('o')) => {
            if let Some(rel) = app.history.back() {
                app.open_rel(&rel);
            }
        }
        #[cfg(feature = "notebook")]
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
    let Some(panel) = app.ui.panel.as_mut() else {
        return;
    };
    match key {
        Some(Key::Char('j') | Key::Char('w') | Key::Char('W')) | Some(Key::Down) => {
            panel.move_down();
        }
        Some(Key::Char('k') | Key::Char('b') | Key::Char('B')) | Some(Key::Up) => panel.move_up(),
        Some(Key::Esc) | Some(Key::Ctrl('w')) => app.ui.panel_focus = false,
        Some(Key::Enter) => match panel.activate() {
            Activation::Open(path) => app.open_rel(&path),
            Activation::Run { command, item } => {
                app.ui.panel = None;
                app.ui.panel_focus = false;
                requests::dispatch(app, &command, &item);
            }
            Activation::Toggled | Activation::None => {}
        },
        _ => {}
    }
}

pub fn buffer_list_key(app: &mut App, event: KeyEvent) {
    let Some(list) = app.ui.buffer_list.as_mut() else {
        return;
    };
    match event_map::to_key(event) {
        Some(Key::Esc) => app.ui.buffer_list = None,
        Some(Key::Enter) => {
            let target = list.current().map(|e| e.idx);
            app.ui.buffer_list = None;
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
                    app.ui.buffer_list = None;
                } else if let Some(list) = app.ui.buffer_list.as_mut() {
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
    let Some(picker) = app.ui.picker.as_mut() else {
        return;
    };
    match event_map::to_key(event) {
        Some(Key::Esc) => app.ui.picker = None,
        Some(Key::Enter) => {
            let chosen = picker
                .current()
                .map(|item| (picker.command().to_string(), item));
            app.ui.picker = None;
            if let Some((command, item)) = chosen {
                requests::dispatch(app, &command, &item);
            }
        }
        Some(Key::Backspace) => picker.pop(),
        Some(Key::Ctrl('n')) | Some(Key::Down) => picker.move_down(),
        Some(Key::Ctrl('p')) | Some(Key::Up) => picker.move_up(),
        Some(Key::Char(c)) => picker.push(c),
        _ => {}
    }
}
