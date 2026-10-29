use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use v_ide_keys::{Code, FKey, Key, Mods, Stroke};

pub fn to_stroke(event: KeyEvent) -> Option<Stroke> {
    let held = event.modifiers;
    let mut mods = Mods {
        ctrl: held.contains(KeyModifiers::CONTROL),
        alt: held.contains(KeyModifiers::ALT),
        shift: held.contains(KeyModifiers::SHIFT),
        sup: held.contains(KeyModifiers::SUPER),
    };
    let code = match event.code {
        KeyCode::Char(c) => {
            mods.shift &= c.is_ascii_alphabetic();
            Code::Char(c)
        }
        KeyCode::BackTab => {
            mods.shift = true;
            Code::Tab
        }
        KeyCode::F(n) => Code::F(FKey::new(n)?),
        KeyCode::Tab => Code::Tab,
        KeyCode::Esc => Code::Esc,
        KeyCode::Enter => Code::Enter,
        KeyCode::Backspace => Code::Backspace,
        KeyCode::Delete => Code::Delete,
        KeyCode::Insert => Code::Insert,
        KeyCode::Home => Code::Home,
        KeyCode::End => Code::End,
        KeyCode::PageUp => Code::PageUp,
        KeyCode::PageDown => Code::PageDown,
        KeyCode::Up => Code::Up,
        KeyCode::Down => Code::Down,
        KeyCode::Left => Code::Left,
        KeyCode::Right => Code::Right,
        _ => return None,
    };
    Some(Stroke {
        key: Key::new(code, mods),
        base: None,
    })
}
