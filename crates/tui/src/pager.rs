use v_ide_keys::{Code, Key, Mods};

use crate::message_wrap::{self, WrappedRow};

#[derive(Debug, PartialEq, Eq)]
pub enum PagerInput {
    Keep,
    Close,
    Replay(Key),
    Command,
}

pub struct Pager {
    text: String,
    rows: Vec<WrappedRow>,
    top: usize,
    width: u16,
    height: u16,
    asking_more: bool,
    help: bool,
}

impl Pager {
    pub fn new(text: impl Into<String>, width: u16, height: u16) -> Self {
        let text = message_wrap::normalize(&text.into());
        let rows = message_wrap::wrap(&text, width);
        let asking_more = rows.len() > usize::from(height.saturating_sub(1).max(1));
        Self {
            text,
            rows,
            top: 0,
            width,
            height,
            asking_more,
            help: false,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn visible(&self) -> impl Iterator<Item = &str> {
        self.rows
            .iter()
            .skip(self.top)
            .take(self.capacity())
            .map(|row| row.text.as_str())
    }

    pub fn more(&self) -> bool {
        self.asking_more
    }

    pub fn prompt(&self) -> &'static str {
        if self.more() {
            if self.help {
                "-- More -- SPACE/d/j: screen/page/line down, b/u/k: up, q: quit"
            } else {
                "-- More --"
            }
        } else {
            "Press ENTER or type command to continue"
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width {
            let anchor = self.rows.get(self.top).map_or(0, |row| row.start);
            self.rows = message_wrap::wrap(&self.text, width);
            self.top = self
                .rows
                .partition_point(|row| row.start <= anchor)
                .saturating_sub(1);
            self.width = width;
        }
        self.height = height;
        self.top = self.top.min(self.last_top());
        if self.top + self.capacity() < self.rows.len() {
            self.asking_more = true;
        }
    }

    pub fn key(&mut self, key: Key) -> PagerInput {
        self.help = false;
        let backward = match (key.mods(), key.code()) {
            (Mods::NONE, Code::Char('k') | Code::Up) => Some(1),
            (Mods::NONE, Code::Backspace) if self.more() => Some(1),
            (Mods::NONE, Code::Char('u')) => Some(usize::from(self.height / 2).max(1)),
            (Mods::NONE, Code::Char('b') | Code::PageUp) | (Mods::CTRL, Code::Char('b')) => {
                Some(self.capacity())
            }
            _ => None,
        };
        if let Some(rows) = backward {
            if self.more() || self.rows.len() > usize::from(self.height) {
                self.top = self.top.saturating_sub(rows.max(1));
                self.asking_more = true;
            }
            return PagerInput::Keep;
        }
        if key == Key::char('g') {
            if self.more() || self.rows.len() > usize::from(self.height) {
                self.top = 0;
                self.asking_more = true;
            }
            return PagerInput::Keep;
        }
        match (key.mods(), key.code()) {
            (Mods::NONE, Code::Char('q')) | (Mods::CTRL, Code::Char('c')) => PagerInput::Close,
            (Mods::NONE, Code::Esc) if self.more() => PagerInput::Close,
            (Mods::NONE, Code::Char(':')) => PagerInput::Command,
            (Mods::NONE, Code::Enter | Code::Char(' ')) | (Mods::CTRL, Code::Char('j'))
                if !self.more() =>
            {
                PagerInput::Close
            }
            (Mods::NONE, Code::Char('j' | 'd' | 'f') | Code::Down | Code::PageDown)
            | (Mods::CTRL, Code::Char('f'))
                if !self.more() && self.rows.len() > self.capacity() =>
            {
                PagerInput::Keep
            }
            _ if !self.more() => PagerInput::Replay(key),
            (Mods::NONE, Code::Enter | Code::Down | Code::Char('j'))
            | (Mods::CTRL, Code::Char('j')) => self.down(1),
            (Mods::NONE, Code::Char('d')) => self.down(usize::from(self.height / 2).max(1)),
            (Mods::NONE, Code::Char(' ' | 'f') | Code::PageDown)
            | (Mods::CTRL, Code::Char('f')) => self.down(self.capacity()),
            (Mods::NONE, Code::Char('G')) => {
                self.top = self.last_top();
                self.asking_more = false;
                PagerInput::Keep
            }
            _ => {
                self.help = true;
                PagerInput::Keep
            }
        }
    }

    fn capacity(&self) -> usize {
        usize::from(self.height.saturating_sub(1).max(1))
    }
    fn last_top(&self) -> usize {
        self.rows.len().saturating_sub(self.capacity())
    }
    fn down(&mut self, rows: usize) -> PagerInput {
        if rows > self.last_top().saturating_sub(self.top) {
            self.asking_more = false;
        }
        self.top = self.top.saturating_add(rows).min(self.last_top());
        PagerInput::Keep
    }
}
