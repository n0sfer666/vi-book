use crate::app::App;
use crate::disk_message;
use v_ide_editor::workspace::DocId;

pub fn run(app: &mut App) -> bool {
    let mut events = Vec::new();
    let mut changed = false;
    let held = app
        .ui
        .read_select
        .is_some()
        .then(|| app.workspace.current_id());
    for id in ids(app) {
        if Some(id) == held {
            continue;
        }
        let Some(doc) = app.workspace.doc_mut(id) else {
            continue;
        };
        let before = doc.disk_mark();
        if let Some(reaction) = doc.check_disk() {
            events.push((doc.name.clone(), reaction));
        }
        changed |= doc.disk_mark() != before;
    }
    if let Some(text) = disk_message::summary(&events) {
        app.ui.message = text;
        changed = true;
    }
    changed
}

pub fn sync_twins(app: &mut App) {
    let current = app.workspace.current_id();
    let Some(path) = app.workspace.current().path.clone() else {
        return;
    };
    for id in ids(app) {
        let Some(doc) = app.workspace.doc_mut(id) else {
            continue;
        };
        if id != current && doc.path.as_ref() == Some(&path) {
            doc.check_disk();
        }
    }
}

fn ids(app: &App) -> Vec<DocId> {
    app.workspace
        .windows()
        .iter()
        .map(|w| w.buf.id)
        .chain(app.workspace.pool().iter().map(|b| b.id))
        .collect()
}
