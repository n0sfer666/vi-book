use crate::core::buffer::Buffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    Word,
    Punct,
    Blank,
}

pub fn class(c: char) -> CharClass {
    if c.is_whitespace() {
        CharClass::Blank
    } else if c.is_alphanumeric() || c == '_' {
        CharClass::Word
    } else {
        CharClass::Punct
    }
}

fn class_at(buffer: &Buffer, idx: usize) -> Option<CharClass> {
    buffer.char_at(idx).map(class)
}

pub fn next_word_start(buffer: &Buffer, idx: usize) -> usize {
    let len = buffer.len_chars();
    let mut j = idx;
    if let Some(c) = class_at(buffer, j)
        && c != CharClass::Blank
    {
        while class_at(buffer, j) == Some(c) {
            j += 1;
        }
    }
    while class_at(buffer, j) == Some(CharClass::Blank) {
        if buffer.char_at(j) == Some('\n') && buffer.char_at(j + 1) == Some('\n') {
            return j + 1;
        }
        j += 1;
    }
    j.min(len)
}

pub fn next_word_end(buffer: &Buffer, idx: usize) -> usize {
    let len = buffer.len_chars();
    if len == 0 {
        return 0;
    }
    let mut j = (idx + 1).min(len - 1);
    while class_at(buffer, j) == Some(CharClass::Blank) {
        if j + 1 >= len {
            return len - 1;
        }
        j += 1;
    }
    if let Some(c) = class_at(buffer, j) {
        while class_at(buffer, j + 1) == Some(c) {
            j += 1;
        }
    }
    j
}

pub fn prev_word_start(buffer: &Buffer, idx: usize) -> usize {
    if idx == 0 {
        return 0;
    }
    let mut j = idx - 1;
    while j > 0 && class_at(buffer, j) == Some(CharClass::Blank) {
        j -= 1;
    }
    let Some(c) = class_at(buffer, j) else {
        return 0;
    };
    if c == CharClass::Blank {
        return 0;
    }
    while j > 0 && class_at(buffer, j - 1) == Some(c) {
        j -= 1;
    }
    j
}

pub fn word_run(buffer: &Buffer, idx: usize) -> Option<(usize, usize)> {
    let c = class_at(buffer, idx)?;
    if buffer.char_at(idx) == Some('\n') {
        return None;
    }
    let mut start = idx;
    while start > 0
        && buffer.char_at(start - 1) != Some('\n')
        && class_at(buffer, start - 1) == Some(c)
    {
        start -= 1;
    }
    let mut end = idx;
    while buffer.char_at(end + 1).is_some_and(|n| n != '\n') && class_at(buffer, end + 1) == Some(c)
    {
        end += 1;
    }
    Some((start, end))
}
