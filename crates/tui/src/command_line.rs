use std::io::ErrorKind;

use crate::app::App;
#[cfg(all(feature = "notebook", feature = "vault"))]
use crate::panel::Panel;
use crate::{actions, disk_message, requests};
use v_ide_editor::Error;
use v_ide_editor::disk::{DiskMark, Reaction};
use v_ide_editor::execution::Error as ExecError;
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
    crate::messages::batch(app, |app| {
        if !execute(app, command) {
            app.typeahead.flush_mapped();
        }
    });
}

fn execute(app: &mut App, command: &str) -> bool {
    if let Some(success) = crate::map_exec::execute(app, command) {
        return success;
    }
    if let Some(success) = crate::set_exec::execute(app, command) {
        return success;
    }
    if let Some(success) = crate::core_exec::execute(app, command) {
        return success;
    }
    match command {
        "w" => return save(app, false),
        "w!" => return save(app, true),
        "q" => return crate::autocmd::quit(app, false),
        "q!" => return crate::autocmd::quit(app, true),
        "wq" => {
            return save(app, false) && crate::autocmd::quit(app, true);
        }
        "e!" => reload(app),
        #[cfg(all(feature = "notebook", feature = "vault"))]
        "tree" => toggle_tree(app),
        "help" => app.ui.help = true,
        "messages" => {
            let text = app.ui.messages.history().collect::<Vec<_>>().join("\n");
            app.ui.show_output(text);
        }
        "messages clear" => {
            app.ui.messages.clear();
            app.ui.show_output("");
        }
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
                app.ui
                    .set_message(format!("not an editor command: {other}"));
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
    let id = app.workspace.current_id();
    let _ = crate::core_exec::with_after(
        app,
        |cx| cx.reload_current(),
        |app, result| {
            let Some(doc) = app.workspace.doc(id) else {
                return;
            };
            let message = match result {
                Ok(()) => disk_message::message(&doc.name, Reaction::Reload),
                Err(ExecError::File(Error::Io(err))) if err.kind() == ErrorKind::NotFound => {
                    match doc.disk_mark() {
                        Some(DiskMark::Deleted) => {
                            disk_message::message(&doc.name, Reaction::Vanished)
                        }
                        _ => disk_message::missing(&doc.name),
                    }
                }
                Err(ExecError::Autocmd(_)) => return,
                Err(error) => format!("reload failed: {error}"),
            };
            app.ui.set_message(message);
        },
    );
}

fn save(app: &mut App, force: bool) -> bool {
    let doc = app.workspace.current_mut();
    if !force {
        doc.check_disk();
    }
    if !force && doc.disk_mark() == Some(DiskMark::Changed) {
        app.ui
            .set_message(disk_message::message(&doc.name, Reaction::Conflict));
        return false;
    }
    let result = crate::core_exec::with_after(
        app,
        |cx| cx.write_current(force),
        |app, result| {
            if result.is_ok() {
                app.ui.set_message(String::from("written"));
            }
        },
    );
    match result {
        Ok(()) => true,
        Err(v_ide_editor::execution::Error::Autocmd(_)) => false,
        Err(err) => {
            app.ui.set_message(format!("save failed: {err}"));
            false
        }
    }
}
