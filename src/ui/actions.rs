use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ui::app::App;
use crate::ui::menu::Menu;
use crate::ui::view::View;
use crate::ui::{command_line, help, layout, read_view};
use crate::vim::editor::Editor;
use crate::vim::key::Key;

pub fn open_menu(app: &mut App) {
    app.menu = Some(Menu::leader(app.menu_extra.clone(), app.menu_center));
}

pub fn help_key(app: &mut App, event: KeyEvent) {
    let shift = event.modifiers.contains(KeyModifiers::SHIFT);
    match event.code {
        KeyCode::Enter if shift => open_doc(app, "developers.md", help::DEVELOPERS),
        KeyCode::Enter => open_doc(app, "manual.md", help::MANUAL),
        KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?') | KeyCode::Char('q') => app.help = false,
        _ => {}
    }
}

pub fn menu_key(app: &mut App, mapped: Option<Key>) {
    let Some(menu) = app.menu.as_mut() else {
        return;
    };
    match mapped {
        Some(Key::Down) | Some(Key::Char('j')) => menu.move_down(),
        Some(Key::Up) | Some(Key::Char('k')) => menu.move_up(),
        Some(Key::Esc) => app.menu = None,
        Some(Key::Enter) => {
            if let Some(action) = menu.current().map(|i| i.action.clone()) {
                dispatch(app, &action);
            }
        }
        Some(Key::Char(c)) => {
            if let Some(action) = menu.by_key(c).map(|i| i.action.clone()) {
                dispatch(app, &action);
            }
        }
        _ => {}
    }
}

pub fn dispatch(app: &mut App, action: &str) {
    app.menu = None;
    match action {
        ":pick" => app.open_picker(),
        ":newprompt" => app.command = Some("new ".to_string()),
        ":toggle-view" => toggle_view(app),
        ":help" => app.help = true,
        cmd => command_line::run(app, cmd.trim_start_matches(':')),
    }
}

pub fn toggle_view(app: &mut App) {
    let width = layout::column_width(app.text_area);
    if app.view == View::Read {
        read_view::leave(app, width);
    } else {
        read_view::enter(app, width);
    }
}

pub fn open_doc(app: &mut App, label: &str, content: &str) {
    if app.editor.text() != app.saved_text {
        app.message = String::from("unsaved changes (write with :w or discard)");
        return;
    }
    app.editor = Editor::new(content);
    app.path = label.to_string();
    app.saved_text = content.to_string();
    app.view = View::Read;
    app.read_cursor = (0, 0);
    app.read_cache = None;
    app.help = false;
    app.menu = None;
    app.top = 0;
}
