use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::vim::key::Key;

pub fn to_key(event: KeyEvent) -> Option<Key> {
    match event.code {
        KeyCode::Char(c) if event.modifiers.contains(KeyModifiers::CONTROL) => Some(Key::Ctrl(c)),
        KeyCode::Char(c) => Some(Key::Char(c)),
        KeyCode::Esc => Some(Key::Esc),
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::Backspace => Some(Key::Backspace),
        _ => None,
    }
}
