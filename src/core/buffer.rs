use std::ops::Range;

use ropey::Rope;

use crate::core::position::Position;

pub struct Buffer {
    rope: Rope,
}

impl Buffer {
    pub fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn char_at(&self, idx: usize) -> Option<char> {
        (idx < self.len_chars()).then(|| self.rope.char(idx))
    }

    fn ends_with_newline(&self) -> bool {
        self.len_chars() > 0 && self.rope.char(self.len_chars() - 1) == '\n'
    }

    pub fn line_count(&self) -> usize {
        let lines = self.rope.len_lines();
        if self.ends_with_newline() {
            (lines - 1).max(1)
        } else {
            lines
        }
    }

    pub fn line_start(&self, line: usize) -> usize {
        self.rope.line_to_char(line)
    }

    pub fn line_len(&self, line: usize) -> usize {
        let slice = self.rope.line(line);
        let len = slice.len_chars();
        if len > 0 && slice.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    pub fn line_end_col(&self, line: usize) -> usize {
        self.line_len(line).saturating_sub(1)
    }

    pub fn line_has_newline(&self, line: usize) -> bool {
        self.char_at(self.line_start(line) + self.line_len(line)) == Some('\n')
    }

    pub fn pos_to_char(&self, pos: Position) -> usize {
        self.line_start(pos.line) + pos.col
    }

    pub fn char_to_pos(&self, idx: usize) -> Position {
        let idx = idx.min(self.len_chars());
        let line = self.rope.char_to_line(idx);
        Position::new(line, idx - self.line_start(line))
    }

    pub fn clamp_normal(&self, pos: Position) -> Position {
        let line = pos.line.min(self.line_count() - 1);
        Position::new(line, pos.col.min(self.line_end_col(line)))
    }

    pub fn first_non_blank(&self, line: usize) -> usize {
        let start = self.line_start(line);
        (0..self.line_len(line))
            .find(|&col| !self.rope.char(start + col).is_whitespace())
            .unwrap_or_else(|| self.line_end_col(line))
    }

    pub fn is_blank_line(&self, line: usize) -> bool {
        let start = self.line_start(line);
        (0..self.line_len(line)).all(|col| self.rope.char(start + col).is_whitespace())
    }

    pub fn insert(&mut self, at: usize, text: &str) {
        self.rope.insert(at, text);
    }

    pub fn remove(&mut self, range: Range<usize>) {
        self.rope.remove(range);
    }

    pub fn slice(&self, range: Range<usize>) -> String {
        self.rope.slice(range).to_string()
    }
}
