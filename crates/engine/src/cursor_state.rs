use crate::{Editor, desired_col::DesiredCol};
use v_ide_text::{Position, tracked_position::TrackedPosition};

#[derive(Debug)]
pub struct CursorState {
    position: TrackedPosition,
    desired: DesiredCol,
}

impl CursorState {
    pub fn position(&self) -> Position {
        self.position.get()
    }
}

impl Editor {
    pub fn cursor_state(&mut self) -> CursorState {
        CursorState {
            position: self.buffer.track_position(self.cursor),
            desired: self.desired,
        }
    }

    pub fn restore_cursor_state(&mut self, state: &CursorState) {
        self.set_cursor(state.position());
        self.desired = state.desired;
    }
}
