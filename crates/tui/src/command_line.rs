use std::io::ErrorKind;

use crate::app::App;
#[cfg(all(feature = "notebook", feature = "vault"))]
use crate::panel::Panel;
use crate::{actions, checktime, disk_message, requests};
use v_ide_editor::Error;
use v_ide_editor::disk::{DiskMark, Reaction};
use v_ide_editor::module::Event;
use v_ide_keys::{Code, Key, Mods};

pub fn handle(app: &mut App, key: Key) {
    let Some(command) = app.ui.command.as_mut() else {
        return;
    };
    match (key.mods(), key.code()) {
        (Mods::NONE, Code::Char(c)) => command.push(c),
        (Mods::NONE, Code::Backspace) => {
            if command.pop().is_none() {
                app.ui.command = None;
            }
        }
        (Mods::NONE, Code::Esc) => app.ui.command = None,
        (Mods::NONE, Code::Enter) => {
            let command = app.ui.command.take().unwrap_or_default();
            run(app, &command);
        }
        _ => {}
    }
}

pub fn run(app: &mut App, command: &str) {
    if !execute(app, command) {
        app.typeahead.flush_mapped();
    }
}

fn execute(app: &mut App, command: &str) -> bool {
    match command {
        "w" => return save(app, false),
        "w!" => return save(app, true),
        "q" => return quit(app, false),
        "q!" => return quit(app, true),
        "wq" => {
            let saved = save(app, false);
            if saved {
                app.quit = true;
            }
            return saved;
        }
        "e!" => reload(app),
        #[cfg(all(feature = "notebook", feature = "vault"))]
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
                return false;
            }
        }
    }
    true
}

#[cfg(all(feature = "notebook", feature = "vault"))]
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

fn quit(app: &mut App, force: bool) -> bool {
    if !force && app.any_dirty() {
        app.ui.message = String::from("unsaved changes (use :q! to discard)");
        return false;
    }
    app.quit = true;
    true
}
