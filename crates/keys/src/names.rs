use crate::code::Code;
use crate::fkey::FKey;

const NAMES: [(&str, Code); 20] = [
    ("Esc", Code::Esc),
    ("CR", Code::Enter),
    ("Enter", Code::Enter),
    ("Return", Code::Enter),
    ("BS", Code::Backspace),
    ("Tab", Code::Tab),
    ("Del", Code::Delete),
    ("Insert", Code::Insert),
    ("Home", Code::Home),
    ("End", Code::End),
    ("PageUp", Code::PageUp),
    ("PageDown", Code::PageDown),
    ("Up", Code::Up),
    ("Down", Code::Down),
    ("Left", Code::Left),
    ("Right", Code::Right),
    ("Space", Code::Char(' ')),
    ("lt", Code::Char('<')),
    ("Bar", Code::Char('|')),
    ("Bslash", Code::Char('\\')),
];

pub fn code_by_name(name: &str) -> Option<Code> {
    NAMES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(name))
        .map(|&(_, code)| code)
        .or_else(|| function_key(name))
}

pub fn name_of(code: Code) -> Option<String> {
    match code {
        Code::F(key) => Some(format!("F{}", key.n())),
        _ => NAMES
            .iter()
            .find(|&&(_, known)| known == code)
            .map(|(name, _)| name.to_string()),
    }
}

fn function_key(name: &str) -> Option<Code> {
    let digits = name.strip_prefix(['F', 'f'])?;
    let n: u8 = digits.parse().ok()?;
    FKey::new(n)
        .filter(|_| n.to_string() == digits)
        .map(Code::F)
}
