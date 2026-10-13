use ratatui::layout::Rect;

use crate::buffer_list::BufferList;
use crate::menu::Menu;
use crate::panel::Panel;
use crate::picker::Picker;
use crate::read_state::ReadState;
use crate::view::View;
use v_ide_engine::mode::Mode;

pub struct UiState {
    pub read: ReadState,
    pub left: usize,
    pub command: Option<String>,
    pub message: String,
    pub text_area: Rect,
    pub picker: Option<Picker>,
    pub panel: Option<Panel>,
    pub panel_focus: bool,
    pub menu: Option<Menu>,
    pub help: bool,
    pub buffer_list: Option<BufferList>,
    pub pending_ctrl_w: bool,
    pub read_select: Option<(Mode, (usize, usize))>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            read: ReadState {
                view: View::Edit,
                ..ReadState::new()
            },
            left: 0,
            command: None,
            message: String::new(),
            text_area: Rect::new(0, 0, 80, 24),
            picker: None,
            panel: None,
            panel_focus: false,
            menu: None,
            help: false,
            buffer_list: None,
            pending_ctrl_w: false,
            read_select: None,
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
