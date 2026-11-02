use crate::{Editor, Mode, desired_col::DesiredCol};
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
        let position = state.position();
        self.cursor = if self.mode == Mode::Insert {
            let line = position.line.min(self.buffer.line_count() - 1);
            Position::new(line, position.col.min(self.buffer.line_len(line)))
        } else {
            self.buffer.clamp_normal(position)
        };
        self.desired = state.desired;
    }
}
