use crate::view::View;
use v_ide_markdown::rendered::Rendered;
use v_ide_markdown::wrap_mode::WrapMode;

pub struct ReadState {
    pub top: usize,
    pub view: View,
    pub read_cursor: (usize, usize),
    pub read_pending_g: bool,
    pub read_cache: Option<(String, usize, WrapMode, Rendered)>,
}

impl ReadState {
    pub fn new() -> Self {
        Self {
            top: 0,
            view: View::Read,
            read_cursor: (0, 0),
            read_pending_g: false,
            read_cache: None,
        }
    }
}

impl Default for ReadState {
    fn default() -> Self {
        Self::new()
    }
}
