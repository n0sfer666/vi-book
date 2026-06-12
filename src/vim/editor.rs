use std::path::{Path, PathBuf};

use crate::core::buffer::Buffer;
use crate::core::edit::Edit;
use crate::core::history::History;
use crate::core::position::Position;
use crate::core::transaction::Transaction;
use crate::io::error::Error;
use crate::io::file;
use crate::vim::desired_col::DesiredCol;
use crate::vim::key::{Key, parse_keys};
use crate::vim::mode::Mode;
use crate::vim::pending::{Pending, Stage};
use crate::vim::register::Registers;
use crate::vim::search::SearchState;
use crate::vim::{insert, normal, search, visual};

pub struct Editor {
    pub(crate) buffer: Buffer,
    pub(crate) cursor: Position,
    pub(crate) desired: DesiredCol,
    pub(crate) mode: Mode,
    pub(crate) pending: Pending,
    pub(crate) registers: Registers,
    pub(crate) history: History,
    pub(crate) insert_edits: Vec<Edit>,
    pub(crate) visual_anchor: Position,
    pub(crate) search: SearchState,
    path: Option<PathBuf>,
}

impl Editor {
    pub fn new(text: &str) -> Self {
        Self {
            buffer: Buffer::new(text),
            cursor: Position::default(),
            desired: DesiredCol::default(),
            mode: Mode::Normal,
            pending: Pending::default(),
            registers: Registers::default(),
            history: History::default(),
            insert_edits: Vec::new(),
            visual_anchor: Position::default(),
            search: SearchState::default(),
            path: None,
        }
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let text = file::read(path.as_ref())?;
        let mut editor = Self::new(&text);
        editor.path = Some(path.as_ref().to_path_buf());
        Ok(editor)
    }

    pub fn save(&mut self) -> Result<(), Error> {
        let path = self.path.as_ref().ok_or(Error::NoPath)?;
        file::write(path, &self.buffer.text())
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    pub fn cursor(&self) -> Position {
        self.cursor
    }

    pub fn set_cursor(&mut self, pos: Position) {
        self.cursor = self.buffer.clamp_normal(pos);
        self.sync_desired();
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn handle_key(&mut self, key: Key) {
        if self.search.input.is_some() {
            search::handle_input(self, key);
            return;
        }
        match self.mode {
            Mode::Normal => normal::handle(self, key),
            Mode::Insert => insert::handle(self, key),
            Mode::Visual | Mode::VisualLine => visual::handle(self, key),
        }
    }

    pub fn handle_keys(&mut self, notation: &str) {
        for key in parse_keys(notation) {
            self.handle_key(key);
        }
    }

    pub fn search_input(&self) -> Option<&str> {
        self.search.input.as_deref()
    }

    pub fn text_index(&self) -> usize {
        self.cursor_index()
    }

    pub fn set_cursor_index(&mut self, idx: usize) {
        self.set_cursor(self.buffer.char_to_pos(idx));
    }

    pub fn idle_normal(&self) -> bool {
        self.mode == Mode::Normal
            && self.pending.stage == Stage::Start
            && self.pending.count.is_none()
            && self.search.input.is_none()
    }

    pub(crate) fn cursor_index(&self) -> usize {
        self.buffer.pos_to_char(self.cursor)
    }

    pub(crate) fn sync_desired(&mut self) {
        self.desired = DesiredCol::Col(self.cursor.col);
    }

    pub(crate) fn commit(&mut self, edits: Vec<Edit>) {
        if edits.is_empty() {
            return;
        }
        self.history.record(Transaction {
            edits,
            cursor_after: self.cursor,
        });
    }

    pub(crate) fn undo(&mut self) {
        if let Some(cursor) = self.history.undo(&mut self.buffer) {
            self.cursor = self.buffer.clamp_normal(cursor);
            self.sync_desired();
        }
    }

    pub(crate) fn redo(&mut self) {
        if let Some(cursor) = self.history.redo(&mut self.buffer) {
            self.cursor = self.buffer.clamp_normal(cursor);
            self.sync_desired();
        }
    }
}
