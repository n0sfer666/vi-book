use crossterm::event::KeyEvent;

use crate::app::App;
use crate::view::View;
use crate::{actions, clip_sync, command_line, event_map, input, layout, read_view};
use v_ide_keys::{Code, Key, Mods};

pub fn handle(app: &mut App, key: KeyEvent) {
    app.ui.message.clear();
    if app.ui.help {
        actions::help_key(app, key);
        return;
    }
    if app.ui.buffer_list.is_some() {
        input::buffer_list_key(app, key);
        return;
    }
    if app.ui.picker.is_some() {
        input::picker_key(app, key);
        return;
    }
    if app.ui.command.is_some() {
        command_line::handle(app, key);
        return;
    }
    let mapped = event_map::to_stroke(key).map(|stroke| stroke.key);
    if app.ui.pending_ctrl_w {
        app.ui.pending_ctrl_w = false;
        window_command(app, mapped);
        return;
    }
    if app.ui.menu.is_some() {
        actions::menu_key(app, mapped);
        return;
    }
    if mapped.is_some_and(actions::is_help_key) {
        app.ui.help = true;
        return;
    }
    if app.ui.panel_focus && app.ui.panel.is_some() {
        input::panel_key(app, mapped);
        return;
    }
    if app.workspace.current().editor.idle_normal() {
        if mapped == Some(Key::char(app.leader)) {
            actions::open_menu(app);
            return;
        }
        if mapped == Some(Key::char('?')) {
            app.ui.help = true;
            return;
        }
    }
    if (app.ui.read.view == View::Read || app.workspace.current().editor.idle_normal())
        && mapped == Some(Key::ctrl('w'))
    {
        app.ui.pending_ctrl_w = true;
        return;
    }
    let width = layout::column_width(app.ui.text_area);
    if app.ui.read.view == View::Read {
        if !input::global_nav(app, mapped) {
            read_view::handle_key(app, key, width);
        }
        return;
    }
    if app.workspace.current().editor.idle_normal() && input::normal_builtin(app, mapped) {
        return;
    }
    #[cfg(feature = "lua")]
    if app.workspace.current().editor.idle_normal()
        && let Some(rhs) = mapped.and_then(|k| app.normal_maps.get(&k).cloned())
    {
        crate::lua_bridge::apply_keymap(app, &rhs);
        return;
    }
    if app.workspace.current().editor.idle_normal() && mapped == Some(Key::ctrl('t')) {
        read_view::enter(app, width);
        return;
    }
    if let Some(key) = mapped {
        clip_sync::editor_key(app, key);
    }
}

fn window_command(app: &mut App, mapped: Option<Key>) {
    match mapped.map(|k| (k.mods(), k.code())) {
        Some((Mods::NONE, Code::Char('w'))) | Some((Mods::CTRL, Code::Char('w'))) => {
            app.focus_next()
        }
        Some((Mods::NONE, Code::Char('p'))) => {
            if app.ui.panel.is_some() {
                app.ui.panel_focus = true;
            }
        }
        Some((Mods::NONE, Code::Char('s' | 'v'))) => app.vsplit(None),
        Some((Mods::NONE, Code::Char('o'))) => app.only(),
        Some((Mods::NONE, Code::Char('c'))) => app.close_active(),
        _ => {}
    }
}
