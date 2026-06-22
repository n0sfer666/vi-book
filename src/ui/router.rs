use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::app::App;
use crate::ui::view::View;
use crate::ui::{actions, command_line, event_map, input, layout, plugin, read_view};
use crate::vim::key::Key;

pub fn handle(app: &mut App, key: KeyEvent) {
    app.message.clear();
    if app.help {
        actions::help_key(app, key);
        return;
    }
    if app.buffer_list.is_some() {
        input::buffer_list_key(app, key);
        return;
    }
    if app.picker.is_some() {
        input::picker_key(app, key);
        return;
    }
    if app.command.is_some() {
        command_line::handle(app, key);
        return;
    }
    let mapped = event_map::to_key(key);
    if app.pending_ctrl_w {
        app.pending_ctrl_w = false;
        window_command(app, mapped);
        return;
    }
    if app.menu.is_some() {
        actions::menu_key(app, mapped);
        return;
    }
    if key.code == KeyCode::F(1) {
        app.help = true;
        return;
    }
    if app.panel_focus && app.panel.is_some() {
        input::panel_key(app, mapped);
        return;
    }
    if app.editor.idle_normal() {
        if mapped == Some(Key::Char(app.leader)) {
            actions::open_menu(app);
            return;
        }
        if mapped == Some(Key::Char('?')) {
            app.help = true;
            return;
        }
    }
    if (app.view == View::Read || app.editor.idle_normal()) && mapped == Some(Key::Ctrl('w')) {
        app.pending_ctrl_w = true;
        return;
    }
    let width = layout::column_width(app.text_area);
    if app.view == View::Read {
        if !input::global_nav(app, mapped) {
            read_view::handle_key(app, key, width);
        }
        return;
    }
    if app.editor.idle_normal() && input::normal_builtin(app, mapped) {
        return;
    }
    if app.editor.idle_normal()
        && let Some(rhs) = mapped.and_then(|k| app.normal_maps.get(&k).cloned())
    {
        plugin::apply_keymap(app, &rhs);
        return;
    }
    if app.editor.idle_normal() && mapped == Some(Key::Ctrl('t')) {
        read_view::enter(app, width);
        return;
    }
    if let Some(key) = mapped {
        app.editor.handle_key(key);
    }
}

fn window_command(app: &mut App, mapped: Option<Key>) {
    match mapped {
        Some(Key::Char('w')) | Some(Key::Ctrl('w')) => app.focus_next(),
        Some(Key::Char('p')) => {
            if app.panel.is_some() {
                app.panel_focus = true;
            }
        }
        Some(Key::Char('s')) | Some(Key::Char('v')) => app.vsplit(None),
        Some(Key::Char('o')) => app.only(),
        Some(Key::Char('c')) => app.close_active(),
        _ => {}
    }
}
