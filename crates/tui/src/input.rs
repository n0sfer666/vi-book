use crossterm::event::KeyEvent;

use crate::app::App;
use crate::panel::Activation;
use crate::{event_map, link_follow, requests};
use v_ide_keys::{Code, Key, Mods};

pub fn global_nav(app: &mut App, mapped: Option<Key>) -> bool {
    match mapped.map(|k| (k.mods(), k.code())) {
        Some((Mods::NONE, Code::Char(':'))) => app.ui.command = Some(String::new()),
        #[cfg(feature = "vault")]
        Some((Mods::CTRL, Code::Char('p'))) => app.open_picker(),
        #[cfg(feature = "notebook")]
        Some((Mods::CTRL, Code::Char('o'))) => {
            if let Some(rel) = app.history.back() {
                app.open_rel(&rel);
            }
        }
        #[cfg(feature = "notebook")]
        Some((Mods::NONE, Code::Tab)) => {
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
    match mapped.map(|k| (k.mods(), k.code())) {
        Some((Mods::NONE, Code::Enter)) => link_follow::follow_at_cursor(app),
        _ => return false,
    }
    true
}

pub fn panel_key(app: &mut App, key: Option<Key>) {
    let Some(panel) = app.ui.panel.as_mut() else {
        return;
    };
    match key.map(|k| (k.mods(), k.code())) {
        Some((Mods::NONE, Code::Char('j' | 'w' | 'W') | Code::Down)) => {
            panel.move_down();
        }
        Some((Mods::NONE, Code::Char('k' | 'b' | 'B') | Code::Up)) => panel.move_up(),
        Some((Mods::NONE, Code::Esc)) | Some((Mods::CTRL, Code::Char('w'))) => {
            app.ui.panel_focus = false
        }
        Some((Mods::NONE, Code::Enter)) => match panel.activate() {
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
    match event_map::to_stroke(event).map(|s| (s.key.mods(), s.key.code())) {
        Some((Mods::NONE, Code::Esc)) => app.ui.buffer_list = None,
        Some((Mods::NONE, Code::Enter)) => {
            let target = list.current().map(|e| e.idx);
            app.ui.buffer_list = None;
            if let Some(idx) = target
                && idx > 0
            {
                app.switch_to_buffer(idx - 1);
            }
        }
        Some((Mods::CTRL, Code::Char('d'))) => {
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
        Some((Mods::NONE, Code::Backspace)) => list.pop(),
        Some((Mods::CTRL, Code::Char('n'))) | Some((Mods::NONE, Code::Down)) => list.move_down(),
        Some((Mods::CTRL, Code::Char('p'))) | Some((Mods::NONE, Code::Up)) => list.move_up(),
        Some((Mods::NONE, Code::Char(c))) => list.push(c),
        _ => {}
    }
}

pub fn picker_key(app: &mut App, event: KeyEvent) {
    let Some(picker) = app.ui.picker.as_mut() else {
        return;
    };
    match event_map::to_stroke(event).map(|s| (s.key.mods(), s.key.code())) {
        Some((Mods::NONE, Code::Esc)) => app.ui.picker = None,
        Some((Mods::NONE, Code::Enter)) => {
            let chosen = picker
                .current()
                .map(|item| (picker.command().to_string(), item));
            app.ui.picker = None;
            if let Some((command, item)) = chosen {
                requests::dispatch(app, &command, &item);
            }
        }
        Some((Mods::NONE, Code::Backspace)) => picker.pop(),
        Some((Mods::CTRL, Code::Char('n'))) | Some((Mods::NONE, Code::Down)) => picker.move_down(),
        Some((Mods::CTRL, Code::Char('p'))) | Some((Mods::NONE, Code::Up)) => picker.move_up(),
        Some((Mods::NONE, Code::Char(c))) => picker.push(c),
        _ => {}
    }
}
