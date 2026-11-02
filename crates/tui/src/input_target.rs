use crate::app::App;
use crate::view::View;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InputTarget {
    Pager,
    Help,
    BufferList,
    Picker,
    Command,
    Window,
    Menu,
    Panel,
    Read,
    Editor,
}

pub(crate) fn target(app: &App) -> InputTarget {
    let ui = &app.ui;
    if ui.pager.is_some() {
        InputTarget::Pager
    } else if ui.help {
        InputTarget::Help
    } else if ui.buffer_list.is_some() {
        InputTarget::BufferList
    } else if ui.picker.is_some() {
        InputTarget::Picker
    } else if ui.command.is_some() {
        InputTarget::Command
    } else if ui.pending_ctrl_w {
        InputTarget::Window
    } else if ui.menu.is_some() {
        InputTarget::Menu
    } else if ui.panel_focus && ui.panel.is_some() {
        InputTarget::Panel
    } else if ui.read.view == View::Read {
        InputTarget::Read
    } else {
        InputTarget::Editor
    }
}
