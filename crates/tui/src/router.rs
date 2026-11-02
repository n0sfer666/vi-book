use crate::app::App;
use crate::input_target::{InputTarget, target};
use crate::{actions, clip_sync, command_line, input, layout, read_view};
use v_ide_keys::{Code, Key, MapMode, Mods};

pub fn handle(app: &mut App, key: Key) {
    match target(app) {
        InputTarget::Pager => crate::message_input::handle(app, key),
        InputTarget::Help => actions::help_key(app, key),
        InputTarget::BufferList => input::buffer_list_key(app, key),
        InputTarget::Picker => input::picker_key(app, key),
        InputTarget::Command => command_line::handle(app, key),
        InputTarget::Window => {
            app.ui.pending_ctrl_w = false;
            window_command(app, key);
        }
        InputTarget::Menu => actions::menu_key(app, key),
        _ if actions::is_help_key(key) => app.ui.help = true,
        InputTarget::Panel => input::panel_key(app, key),
        InputTarget::Read => read_key(app, key),
        InputTarget::Editor => editor_key(app, key),
    }
}

fn read_key(app: &mut App, key: Key) {
    if key == Key::ctrl('w') {
        app.ui.pending_ctrl_w = true;
        return;
    }
    let width = layout::column_width(app.ui.text_area);
    if !input::global_nav(app, key) {
        read_view::handle_key(app, key, width);
    }
}

fn editor_key(app: &mut App, key: Key) {
    let editor = &mut app.workspace.current_mut().editor;
    if key == Key::char(':') && editor.map_mode() == Some(MapMode::Normal) {
        editor.discard_count();
    }
    if app.workspace.current().editor.idle_normal() && idle_key(app, key) {
        return;
    }
    clip_sync::editor_key(app, key);
}

fn idle_key(app: &mut App, key: Key) -> bool {
    if key == Key::char('?') {
        app.ui.help = true;
    } else if key == Key::ctrl('w') {
        app.ui.pending_ctrl_w = true;
    } else if key == Key::ctrl('t') {
        read_view::enter(app, layout::column_width(app.ui.text_area));
    } else {
        return input::normal_builtin(app, key);
    }
    true
}

fn window_command(app: &mut App, key: Key) {
    match (key.mods(), key.code()) {
        (Mods::NONE, Code::Char('w')) | (Mods::CTRL, Code::Char('w')) => app.focus_next(),
        (Mods::NONE, Code::Char('p')) => {
            if app.ui.panel.is_some() {
                app.ui.panel_focus = true;
            }
        }
        (Mods::NONE, Code::Char('s' | 'v')) => app.vsplit(None),
        (Mods::NONE, Code::Char('o')) => app.only(),
        (Mods::NONE, Code::Char('c')) => app.close_active(),
        _ => {}
    }
}
