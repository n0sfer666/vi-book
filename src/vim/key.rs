#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            keys.push(special(&name));
        } else {
            keys.push(Key::Char(c));
        }
    }
    keys
}

fn special(name: &str) -> Key {
    match name {
        "Esc" => Key::Esc,
        "CR" => Key::Enter,
        "BS" => Key::Backspace,
        "lt" => Key::Char('<'),
        _ => match name.strip_prefix("C-").and_then(|rest| rest.chars().next()) {
            Some(c) => Key::Ctrl(c),
            None => panic!("unknown key notation: <{name}>"),
        },
    }
}
