use std::io::ErrorKind;

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::panel::Panel;
use crate::{actions, checktime, disk_message, requests};
use v_ide_editor::Error;
use v_ide_editor::disk::{DiskMark, Reaction};
use v_ide_editor::module::Event;

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
        "w" => _ = save(app, false),
        "w!" => _ = save(app, true),
        "e!" => reload(app),
        "q" => quit(app, false),
        "q!" => quit(app, true),
        "wq" => {
            if save(app, false) {
                app.quit = true;
            }
        }
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
            } else if !requests::command(app, other) {
                app.ui.message = format!("not an editor command: {other}");
            }
        }
    }
}

fn toggle_tree(app: &mut App) {
    if matches!(app.ui.panel, Some(Panel::Tree { .. })) {
        app.ui.panel = None;
        app.ui.panel_focus = false;
        return;
    }
    let items = app.vault_items();
    app.ui.panel = Some(Panel::tree(&items));
    app.ui.panel_focus = true;
}

fn reload(app: &mut App) {
    let doc = app.workspace.current_mut();
    app.ui.message = match doc.reload() {
        Ok(()) => disk_message::message(&doc.name, Reaction::Reload),
        Err(Error::Io(err)) if err.kind() == ErrorKind::NotFound => match doc.disk_mark() {
            Some(DiskMark::Deleted) => disk_message::message(&doc.name, Reaction::Vanished),
            _ => disk_message::missing(&doc.name),
        },
        Err(err) => format!("reload failed: {err}"),
    };
}

fn save(app: &mut App, force: bool) -> bool {
    let doc = app.workspace.current_mut();
    if !force {
        doc.check_disk();
    }
    if !force && doc.disk_mark() == Some(DiskMark::Changed) {
        app.ui.message = disk_message::message(&doc.name, Reaction::Conflict);
        return false;
    }
    match app.workspace.current_mut().save() {
        Ok(()) => {
            app.ui.message = String::from("written");
            checktime::sync_twins(app);
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
