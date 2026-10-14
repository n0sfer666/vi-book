use std::path::Path;

use crate::app::App;
use crate::command_line;
use crate::list_panel::ListPanel;
use crate::panel::Panel;
use crate::picker::Picker;
use v_ide_editor::module::{Event, Request};

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
            true
        }
        None => false,
    }
}

pub fn dispatch(app: &mut App, command: &str, item: &str) {
    match app.modules.command(&app.workspace, command, item) {
        Some(requests) => apply(app, requests),
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
            } => {
                app.ui.panel = Some(Panel::List(ListPanel::new(&title, rows, &command)));
                app.ui.panel_focus = true;
            }
            Request::Emit(ev) => emit_at(app, ev, depth),
        }
    }
}

fn open(app: &mut App, path: &Path) {
    let rel = path
        .strip_prefix(&app.session.root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    app.open_rel(&rel);
}
