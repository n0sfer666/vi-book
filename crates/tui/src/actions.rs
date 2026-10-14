use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::menu::Menu;
use crate::view::View;
use crate::{command_line, help, layout, read_view};
use v_ide_editor::document::Document;
use v_ide_keys::key::Key;
use v_ide_vault::module::VaultModule;

pub fn open_menu(app: &mut App) {
    app.ui.menu = Some(Menu::leader(
        app.menu_extra.clone(),
        app.menu_center,
        app.wrap,
        app.vault().is_some_and(VaultModule::scan_hidden),
    ));
}

pub fn help_key(app: &mut App, event: KeyEvent) {
    let shift = event.modifiers.contains(KeyModifiers::SHIFT);
    match event.code {
        KeyCode::Enter if shift => open_doc(app, "developers.md", help::DEVELOPERS),
        KeyCode::Enter => open_doc(app, "manual.md", help::MANUAL),
        KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?') | KeyCode::Char('q') => {
            app.ui.help = false
        }
        _ => {}
    }
}

pub fn menu_key(app: &mut App, mapped: Option<Key>) {
    let Some(menu) = app.ui.menu.as_mut() else {
        return;
    };
    match mapped {
        Some(Key::Down) | Some(Key::Char('j')) => menu.move_down(),
        Some(Key::Up) | Some(Key::Char('k')) => menu.move_up(),
        Some(Key::Esc) => {
            if !menu.back() {
                app.ui.menu = None;
            }
        }
        Some(Key::Enter) => {
            let idx = menu.selected;
            activate(app, idx);
        }
        Some(Key::Char(c)) => {
            if let Some(idx) = menu.index_of(c) {
                activate(app, idx);
            }
        }
        _ => {}
    }
}

fn activate(app: &mut App, idx: usize) {
    let Some(menu) = app.ui.menu.as_mut() else {
        return;
    };
    if menu.has_children(idx) {
        menu.enter(idx);
    } else if let Some(action) = menu.action_of(idx) {
        dispatch(app, &action);
    }
}

pub fn dispatch(app: &mut App, action: &str) {
    app.ui.menu = None;
    match action {
        ":pick" => app.open_picker(),
        ":buffers" => app.open_buffer_list(),
        ":newprompt" => app.ui.command = Some("new ".to_string()),
        ":toggle-view" => toggle_view(app),
        ":help" => app.ui.help = true,
        cmd => command_line::run(app, cmd.trim_start_matches(':')),
    }
}

pub fn toggle_view(app: &mut App) {
    let width = layout::column_width(app.ui.text_area);
    if app.ui.read.view == View::Read {
        read_view::leave(app, width);
    } else {
        read_view::enter(app, width);
    }
}

pub fn open_doc(app: &mut App, label: &str, content: &str) {
    if app.workspace.current().is_dirty() {
        app.ui.message = String::from("unsaved changes (write with :w or discard)");
        return;
    }
    let mut doc = Document::new(content);
    doc.name = label.to_string();
    let _ = app.workspace.replace(doc);
    app.ui.read.view = View::Read;
    app.ui.read.read_cursor = (0, 0);
    app.ui.read.read_cache = None;
    app.ui.help = false;
    app.ui.menu = None;
    app.ui.read.top = 0;
}
