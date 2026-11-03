use std::path::Path;

use crate::app::App;
use crate::list_panel::ListPanel;
use crate::panel::Panel;
use crate::picker::Picker;
use crate::{checktime, command_line, external};
use v_ide_editor::module::{Event, Request};
use v_ide_editor::path::rel_path;

const MAX_DEPTH: usize = 16;

pub fn startup(app: &mut App) {
    let requests = app.modules.startup(&app.workspace);
    apply(app, requests);
}

pub fn tick(app: &mut App) -> bool {
    crate::messages::batch(app, |app| {
        let requests = app.modules.tick(&app.workspace);
        let changed = !requests.is_empty();
        apply(app, requests);
        checktime::run(app) || changed
    })
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
    intercepts(app, ev);
}

pub(crate) fn intercepts(app: &mut App, ev: Event) -> bool {
    let mut opened = false;
    nested(app, "event", ev.name(), |app| {
        let requests = app.modules.event(&app.workspace, &ev);
        opened = crate::messages::batch(app, |app| apply_inner(app, requests));
    });
    opened
}

pub fn apply(app: &mut App, requests: Vec<Request>) {
    crate::messages::batch(app, |app| {
        apply_inner(app, requests);
    });
}

fn apply_inner(app: &mut App, requests: Vec<Request>) -> bool {
    let mut opened = false;
    for request in requests {
        match request {
            Request::Open(path) => {
                opened = true;
                open(app, &path);
            }
            Request::Execute(action) => {
                let requests = crate::core_exec::with_context(app, |cx| action.run(cx));
                opened |= apply_inner(app, requests);
            }
            Request::Message(msg) => app.ui.set_message(msg),
            Request::Pick { items, command } => {
                app.ui.picker = Some(Picker::new(items, &command));
            }
            Request::List {
                title,
                rows,
                command,
            } => list(app, &title, rows, &command),
            Request::Emit(ev) => emit(app, ev),
            Request::SetText(text) => set_text(app, &text),
            Request::SetOption {
                doc,
                window,
                access,
                change,
            } => match app.workspace.apply_option_at(doc, window, access, &change) {
                Ok(_) => app.sync_timeout(),
                Err(error) => app.ui.set_message(error.to_string()),
            },
            Request::OpenExternal(rel) => {
                opened = true;
                open_external(app, &rel);
            }
            Request::Command(line) => {
                let name = line.split(' ').next().unwrap_or_default().to_owned();
                nested(app, "command", &name, |app| command_line::run(app, &line));
            }
        }
    }
    opened
}

fn nested(app: &mut App, kind: &str, name: &str, run: impl FnOnce(&mut App)) {
    if app.nesting >= MAX_DEPTH {
        app.ui
            .set_message(format!("{kind} loop: {name} exceeded depth {MAX_DEPTH}"));
        return;
    }
    app.nesting += 1;
    run(app);
    app.nesting -= 1;
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

fn set_text(app: &mut App, text: &str) {
    if app.workspace.current().editor.text() != text {
        app.workspace.current_mut().editor.replace_all(text);
    }
}

pub(crate) fn open_external(app: &mut App, rel: &str) {
    match external::resolve(app.workspace.cwd(), rel) {
        Some(path) => app.external_request = Some(path),
        None => app.ui.set_message(format!("file not found: {rel}")),
    }
}

fn open(app: &mut App, path: &Path) {
    let rel = rel_path(app.workspace.cwd(), path);
    app.open_rel(&rel);
}
