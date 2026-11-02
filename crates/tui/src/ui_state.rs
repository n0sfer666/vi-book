use v_ide_view::Rect;

use crate::buffer_list::BufferList;
use crate::menu::Menu;
use crate::messages::Messages;
use crate::pager::Pager;
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
    pub messages: Messages,
    pub pager: Option<Pager>,
    pub screen: Rect,
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
            messages: Messages::default(),
            pager: None,
            screen: Rect {
                x: 0,
                y: 0,
                w: 80,
                h: 24,
            },
            text_area: Rect {
                x: 0,
                y: 0,
                w: 80,
                h: 24,
            },
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

    pub fn set_message(&mut self, text: impl Into<String>) {
        self.output(text.into(), true);
    }

    pub fn show_output(&mut self, text: impl Into<String>) {
        self.output(text.into(), false);
    }

    fn output(&mut self, text: String, remember: bool) {
        if self.messages.output(&text, remember) {
            self.present(text.clone());
        }
        self.message = text;
    }

    pub(crate) fn present(&mut self, text: String) {
        if self.pager.is_none() && self.command.is_none() && !text.is_empty() {
            let pager = Pager::new(text, self.screen.w, self.screen.h);
            if pager.visible().count() > 1 || pager.more() {
                self.pager = Some(pager);
            }
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
