use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::panel::Panel;
use crate::{actions, daily_cmd, plugin, requests};
use v_ide_editor::module::Event;
use v_ide_markdown::wrap_mode::WrapMode;

pub fn handle(app: &mut App, event: KeyEvent) {
    let Some(command) = app.ui.command.as_mut() else {
        return;
    };
    match event.code {
        KeyCode::Char(c) => command.push(c),
        KeyCode::Backspace => {
            if command.pop().is_none() {
                app.ui.command = None;
            }
        }
        KeyCode::Esc => app.ui.command = None,
        KeyCode::Enter => {
            let command = app.ui.command.take().unwrap_or_default();
            run(app, &command);
        }
        _ => {}
    }
}

pub fn run(app: &mut App, command: &str) {
    match command {
        "w" => _ = save(app),
        "q" => quit(app, false),
        "q!" => quit(app, true),
        "wq" => {
            if save(app) {
                app.quit = true;
            }
        }
        "today" => daily_cmd::open_today(app),
        "daily-prev" => daily_cmd::open_prev(app),
        "daily-next" => daily_cmd::open_next(app),
        "tree" => toggle_tree(app),
        "help" => app.ui.help = true,
        "buffers" => app.open_buffer_list(),
        "toggle-view" => actions::toggle_view(app),
        "vsplit" | "vsp" | "split" | "sp" => app.vsplit(None),
        "only" => app.only(),
        "close" => app.close_active(),
        "" => {}
        other => {
            if let Some(name) = other
                .strip_prefix("vsplit ")
                .or_else(|| other.strip_prefix("split "))
            {
                app.vsplit(Some(name.trim()));
            } else if let Some(rel) = other.strip_prefix("e ") {
                app.open_rel(rel.trim());
            } else if let Some(mode) = other.strip_prefix("wrap ") {
                set_wrap(app, mode.trim());
            } else if !requests::command(app, other) && !plugin::run_command(app, other) {
                app.ui.message = format!("not an editor command: {other}");
            }
        }
    }
}

fn set_wrap(app: &mut App, mode: &str) {
    match WrapMode::parse(mode) {
        Some(wrap) => {
            app.wrap = wrap;
            app.ui.read.read_cache = None;
            app.ui.message = format!("wrap: {mode}");
        }
        None => app.ui.message = format!("unknown wrap mode: {mode} (none|text|all)"),
    }
}

fn toggle_tree(app: &mut App) {
    if matches!(app.ui.panel, Some(Panel::Tree { .. })) {
        app.ui.panel = None;
        app.ui.panel_focus = false;
        return;
    }
    app.ui.panel = Some(Panel::tree(&app.vault_items()));
    app.ui.panel_focus = true;
}

fn save(app: &mut App) -> bool {
    match app.workspace.current_mut().save() {
        Ok(()) => {
            app.ui.message = String::from("written");
            plugin::fire(app, "note_save");
            if let Some(path) = app.workspace.current().path.clone() {
                requests::emit(app, Event::NoteSave(path));
            }
            true
        }
        Err(err) => {
            app.ui.message = format!("save failed: {err}");
            false
        }
    }
}

fn quit(app: &mut App, force: bool) {
    if !force && app.any_dirty() {
        app.ui.message = String::from("unsaved changes (use :q! to discard)");
    } else {
        app.quit = true;
    }
}
