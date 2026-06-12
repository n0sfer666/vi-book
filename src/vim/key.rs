#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    Esc,
    Enter,
    Backspace,
    Ctrl(char),
}

pub fn parse_keys(notation: &str) -> Vec<Key> {
    let mut keys = Vec::new();
    let mut chars = notation.chars();
    while let Some(c) = chars.next() {
        if c == '<' {
            let name: String = chars.by_ref().take_while(|&n| n != '>').collect();
            if let Some(key) = special(&name) {
                keys.push(key);
            }
        } else {
            keys.push(Key::Char(c));
        }
    }
    keys
}

fn special(name: &str) -> Option<Key> {
    match name {
        "Esc" => Some(Key::Esc),
        "CR" => Some(Key::Enter),
        "BS" => Some(Key::Backspace),
        "lt" => Some(Key::Char('<')),
        _ => name
            .strip_prefix("C-")
            .and_then(|rest| rest.chars().next())
            .map(Key::Ctrl),
    }
}
