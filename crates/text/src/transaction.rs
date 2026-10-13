use crate::edit::Edit;
use crate::position::Position;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub edits: Vec<Edit>,
    pub cursor_after: Position,
}

impl Transaction {
    pub fn change_start(&self) -> Option<usize> {
        self.edits.first().map(|edit| edit.at)
    }
}
