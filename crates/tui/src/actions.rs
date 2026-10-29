use crossterm::event::KeyEvent;

use crate::app::App;
use crate::menu::Menu;
use crate::view::View;
use crate::{command_line, event_map, help, layout, read_view};
use v_ide_editor::document::Document;
use v_ide_keys::{Code, FKey, Key, Mods};

pub fn open_menu(app: &mut App) {
    app.ui.menu = Some(Menu::leader(
        app.menu_extra.clone(),
        app.menu_center,
        app.wrap(),
    ));
}

pub fn help_key(app: &mut App, event: KeyEvent) {
    let Some(key) = event_map::to_stroke(event).map(|stroke| stroke.key) else {
        return;
    };
    match (key.mods(), key.code()) {
        (Mods::SHIFT, Code::Enter) => open_doc(app, "developers.md", help::DEVELOPERS),
        (Mods::NONE, Code::Enter) => open_doc(app, "manual.md", help::MANUAL),
        (Mods::NONE, Code::Esc | Code::Char('?' | 'q')) => app.ui.help = false,
        _ if is_help_key(key) => app.ui.help = false,
        _ => {}
    }
}

pub fn is_help_key(key: Key) -> bool {
    FKey::new(1).map(|f1| Key::new(Code::F(f1), Mods::NONE)) == Some(key)
}

pub fn menu_key(app: &mut App, mapped: Option<Key>) {
    let Some(menu) = app.ui.menu.as_mut() else {
        return;
    };
    match mapped.map(|k| (k.mods(), k.code())) {
        Some((Mods::NONE, Code::Down | Code::Char('j'))) => menu.move_down(),
        Some((Mods::NONE, Code::Up | Code::Char('k'))) => menu.move_up(),
        Some((Mods::NONE, Code::Esc)) => {
            if !menu.back() {
                app.ui.menu = None;
            }
        }
        Some((Mods::NONE, Code::Enter)) => {
            let idx = menu.selected;
            activate(app, idx);
        }
        Some((Mods::NONE, Code::Char(c))) => {
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
        #[cfg(feature = "vault")]
        ":pick" => app.open_picker(),
        ":buffers" => app.open_buffer_list(),
        #[cfg(feature = "vault")]
        ":newprompt" => app.ui.command = Some("new ".to_string()),
        ":toggle-view" => toggle_view(app),
        ":help" => app.ui.help = true,
        cmd => command_line::run(app, cmd.trim_start_matches(':')),
    }
}

pub fn toggle_view(app: &mut App) {
    if !app.readable() {
        app.ui.message = String::from("read view: markdown only");
        return;
    }
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
