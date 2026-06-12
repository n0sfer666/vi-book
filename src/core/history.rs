use crate::core::buffer::Buffer;
use crate::core::position::Position;
use crate::core::transaction::Transaction;

#[derive(Default)]
pub struct History {
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
}

impl History {
    pub fn record(&mut self, tx: Transaction) {
        self.undo_stack.push(tx);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, buffer: &mut Buffer) -> Option<usize> {
        let tx = self.undo_stack.pop()?;
        for edit in tx.edits.iter().rev() {
            edit.inverted().apply(buffer);
        }
        let start = tx.change_start();
        self.redo_stack.push(tx);
        start
    }

    pub fn redo(&mut self, buffer: &mut Buffer) -> Option<Position> {
        let tx = self.redo_stack.pop()?;
        for edit in &tx.edits {
            edit.apply(buffer);
        }
        let cursor = tx.cursor_after;
        self.undo_stack.push(tx);
        Some(cursor)
    }
}
