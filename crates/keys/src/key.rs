use crate::code::Code;
use crate::mods::Mods;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    code: Code,
    mods: Mods,
}

impl Key {
    pub fn new(code: Code, mods: Mods) -> Key {
        let (code, mods) = match code {
            Code::Char(c) if let Some(letter) = control_letter(c) => {
                ctrl_char(letter, Mods { ctrl: true, ..mods })
            }
            Code::Char(c) if mods.ctrl => ctrl_char(c, mods),
            Code::Char(c) if mods.shift && c.is_ascii_alphabetic() => (
                Code::Char(c.to_ascii_uppercase()),
                Mods {
                    shift: false,
                    ..mods
                },
            ),
            _ => (code, mods),
        };
        Key { code, mods }
    }

    pub fn char(c: char) -> Key {
        Key::new(Code::Char(c), Mods::NONE)
    }

    pub fn ctrl(c: char) -> Key {
        Key::new(Code::Char(c), Mods::CTRL)
    }

    pub fn code(self) -> Code {
        self.code
    }

    pub fn mods(self) -> Mods {
        self.mods
    }

    pub fn plain_char(self) -> Option<char> {
        match (self.mods, self.code) {
            (Mods::NONE, Code::Char(c)) => Some(c),
            _ => None,
        }
    }
}

fn ctrl_char(c: char, mods: Mods) -> (Code, Mods) {
    let alias = match c.to_ascii_lowercase() {
        '[' => Some(Code::Esc),
        'i' => Some(Code::Tab),
        'm' => Some(Code::Enter),
        _ => None,
    };
    match alias {
        Some(code) => (
            code,
            Mods {
                ctrl: false,
                ..mods
            },
        ),
        None => (Code::Char(c.to_ascii_lowercase()), mods),
    }
}

fn control_letter(c: char) -> Option<char> {
    u8::try_from(c)
        .ok()
        .filter(|&byte| byte < 0x20)
        .map(|byte| char::from(byte + b'@'))
}
