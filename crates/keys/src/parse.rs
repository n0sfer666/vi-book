use crate::code::Code;
use crate::key::Key;
use crate::mods::Mods;
use crate::names::code_by_name;

pub fn parse_keys(notation: &str) -> Vec<Key> {
    let chars: Vec<char> = notation.chars().collect();
    let mut keys = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        match bracket(&chars[at..]) {
            Some((key, len)) => {
                keys.push(key);
                at += len;
            }
            None => {
                keys.push(Key::char(chars[at]));
                at += 1;
            }
        }
    }
    keys
}

fn bracket(chars: &[char]) -> Option<(Key, usize)> {
    let rest = chars.strip_prefix(&['<'])?;
    let mut mods = Mods::NONE;
    let mut at = 0;
    while let &[m, '-', _, ..] = &rest[at..] {
        if !add_modifier(&mut mods, m) {
            break;
        }
        at += 2;
    }
    if let &[c, '>', ..] = &rest[at..]
        && mods != Mods::NONE
    {
        return Some((Key::new(Code::Char(c), mods), at + 3));
    }
    let len = rest[at..].iter().position(|&c| c == '>')?;
    let name: String = rest[at..at + len].iter().collect();
    let code = code_by_name(&name)?;
    Some((Key::new(code, mods), at + len + 2))
}

fn add_modifier(mods: &mut Mods, letter: char) -> bool {
    match letter.to_ascii_uppercase() {
        'C' => mods.ctrl = true,
        'M' | 'A' => mods.alt = true,
        'S' => mods.shift = true,
        'D' => mods.sup = true,
        _ => return false,
    }
    true
}
