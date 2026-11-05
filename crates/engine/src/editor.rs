use crate::desired_col::DesiredCol;
use crate::mode::Mode;
use crate::pending::{Pending, Stage};
use crate::register::Registers;
use crate::search::SearchState;
use crate::{insert, normal, search, visual};
use v_ide_keys::{Key, parse_keys};
use v_ide_text::buffer::Buffer;
use v_ide_text::edit::Edit;
use v_ide_text::history::History;
use v_ide_text::position::Position;
use v_ide_text::transaction::Transaction;

pub struct Editor {
    pub(crate) buffer: Buffer,
    pub(crate) cursor: Position,
    pub(crate) desired: DesiredCol,
    pub(crate) mode: Mode,
    pub(crate) pending: Pending,
    pub(crate) registers: Registers,
    pub(crate) history: History,
    pub(crate) insert_edits: Vec<Edit>,
    pub(crate) insert_undo_deferred: bool,
    pub(crate) visual_anchor: Position,
    pub(crate) search: SearchState,
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
            insert_undo_deferred: false,
            visual_anchor: Position::default(),
            search: SearchState::default(),
        }
    }

    pub fn change_tick(&self) -> u64 {
        self.buffer.change_tick()
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    pub fn line_count(&self) -> usize {
        self.buffer.line_count()
    }

    pub fn line_text(&self, line: usize) -> Option<String> {
        (line < self.buffer.line_count()).then(|| {
            let start = self.buffer.line_start(line);
            self.buffer.slice(start..start + self.buffer.line_len(line))
        })
    }

    pub fn cursor(&self) -> Position {
        self.cursor
    }

    pub fn set_cursor(&mut self, pos: Position) {
        self.cursor = if self.mode == Mode::Insert {
            let line = pos.line.min(self.buffer.line_count() - 1);
            Position::new(line, pos.col.min(self.buffer.line_len(line)))
        } else {
            self.buffer.clamp_normal(pos)
        };
        self.sync_desired();
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn visual_anchor(&self) -> Position {
        self.visual_anchor
    }

    pub fn yank(&mut self, text: String, linewise: bool) {
        self.set_unnamed(text, linewise);
        self.registers.note_yank();
    }

    pub fn set_unnamed(&mut self, text: String, linewise: bool) {
        let content = if linewise {
            crate::register::RegisterContent::linewise(text)
        } else {
            crate::register::RegisterContent::charwise(text)
        };
        self.registers.write(None, content);
    }

    pub fn clip_yank_version(&self) -> u64 {
        self.registers.yank_version()
    }

    pub fn pending_register(&self) -> Option<char> {
        self.pending.register
    }

    pub fn yanked(&self) -> Option<String> {
        self.registers
            .read(None)
            .map(|content| content.text.clone())
    }

    pub fn handle_key(&mut self, key: Key) {
        if self.search.input.is_some() {
            search::handle_input(self, key);
            return;
        }
        match self.mode {
            Mode::Normal => normal::handle(self, key),
            Mode::Insert => insert::handle(self, key),
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock => visual::handle(self, key),
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

    pub fn replace_all(&mut self, text: &str) {
        let old = self.buffer.text();
        if old == text {
            return;
        }
        let edit = Edit::replacement(0, old, text.to_string());
        edit.apply(&mut self.buffer);
        if self.mode == Mode::Insert || self.insert_undo_deferred {
            self.insert_edits.push(edit);
        } else {
            self.commit(vec![edit]);
        }
        self.set_cursor(self.cursor);
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
