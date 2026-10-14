use std::fmt;

const CAP: usize = 23;
const REPLACEMENT: &str = "\u{fffd}";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol {
    len: u8,
    bytes: [u8; CAP],
}

impl Symbol {
    pub fn new(text: &str) -> Self {
        let src = if text.len() <= CAP { text } else { REPLACEMENT };
        let mut bytes = [0; CAP];
        bytes[..src.len()].copy_from_slice(src.as_bytes());
        Self {
            len: u8::try_from(src.len()).unwrap_or(0),
            bytes,
        }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..usize::from(self.len)]).unwrap_or(REPLACEMENT)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}
