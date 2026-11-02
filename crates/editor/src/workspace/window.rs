use std::collections::HashMap;
use v_ide_engine::CursorState;
use v_ide_text::Position;

use crate::workspace::{Buffer, BufferRef, DocId, WindowId};

pub struct Window {
    pub id: WindowId,
    pub buf: Buffer,
    pub(super) cursors: HashMap<DocId, CursorState>,
}

pub struct WindowRef<'a> {
    pub id: WindowId,
    pub buf: BufferRef<'a>,
    pub cursor: Position,
}
