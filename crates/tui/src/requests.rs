use std::path::Path;

use crate::app::App;
use crate::list_panel::ListPanel;
use crate::panel::Panel;
use crate::picker::Picker;
use crate::{command_line, plugin};
use v_ide_editor::module::{Event, Request};
use v_ide_vault::scan::rel_path;

const MAX_DEPTH: usize = 16;

pub fn startup(app: &mut App) {
    let requests = app.modules.startup(&app.workspace);
    apply(app, requests);
}

pub fn command(app: &mut App, line: &str) -> bool {
    let (name, args) = line.split_once(' ').unwrap_or((line, ""));
    match app.modules.command(&app.workspace, name, args) {
        Some(requests) => {
            apply(app, requests);
            plugin::sync_vault_host(app);
            true
        }
        None => false,
    }
}

pub fn dispatch(app: &mut App, command: &str, item: &str) {
    match app.modules.command(&app.workspace, command, item) {
        Some(requests) => {
            apply(app, requests);
            plugin::sync_vault_host(app);
        }
        None => command_line::run(app, &format!("{command} {item}")),
    }
}

pub fn emit(app: &mut App, ev: Event) {
    emit_at(app, ev, 0);
}

pub fn apply(app: &mut App, requests: Vec<Request>) {
    apply_at(app, requests, 0);
}

fn emit_at(app: &mut App, ev: Event, depth: usize) {
    if depth >= MAX_DEPTH {
        app.ui.message = format!("event loop: {} exceeded depth {MAX_DEPTH}", ev.name());
        return;
    }
    let requests = app.modules.event(&app.workspace, &ev);
    apply_at(app, requests, depth + 1);
}

fn apply_at(app: &mut App, requests: Vec<Request>, depth: usize) {
    for request in requests {
        match request {
            Request::Open(path) => open(app, &path),
            Request::Message(msg) => app.ui.message = msg,
            Request::Pick { items, command } => {
                app.ui.picker = Some(Picker::new(items, &command));
            }
            Request::List {
                title,
                rows,
                command,
            } => list(app, &title, rows, &command),
            Request::Emit(ev) => emit_at(app, ev, depth),
        }
    }
}

fn list(app: &mut App, title: &str, rows: Vec<String>, command: &str) {
    if matches!(&app.ui.panel, Some(Panel::List(open)) if open.title() == title) {
        app.ui.panel = None;
        app.ui.panel_focus = false;
        return;
    }
    app.ui.panel = Some(Panel::List(ListPanel::new(title, rows, command)));
    app.ui.panel_focus = true;
}

fn open(app: &mut App, path: &Path) {
    let rel = rel_path(app.workspace.cwd(), path);
    app.open_rel(&rel);
}
