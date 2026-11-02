use crate::view::View;
use v_ide_markdown::rendered::Rendered;
use v_ide_markdown::wrap_mode::WrapMode;

pub struct ReadState {
    pub top: usize,
    pub left: usize,
    pub view: View,
    pub read_cursor: (usize, usize),
    pub read_pending_g: bool,
    pub read_cache: Option<(String, usize, WrapMode, Rendered)>,
}

impl ReadState {
    pub fn new() -> Self {
        Self {
            top: 0,
            left: 0,
            view: View::Read,
            read_cursor: (0, 0),
            read_pending_g: false,
            read_cache: None,
        }
    }

    pub(crate) fn fork(&self) -> Self {
        Self {
            top: self.top,
            left: self.left,
            view: self.view,
            read_cursor: self.read_cursor,
            ..Self::new()
        }
    }
}

impl Default for ReadState {
    fn default() -> Self {
        Self::new()
    }
}
