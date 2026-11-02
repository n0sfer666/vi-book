use crate::app::App;
use crate::disk_message;

pub fn run(app: &mut App) -> bool {
    let mut events = Vec::new();
    let mut changed = false;
    let held = app
        .ui
        .read_select
        .is_some()
        .then(|| app.workspace.current_id());
    for id in app.workspace.ids() {
        if Some(id) == held {
            continue;
        }
        let Some(doc) = app.workspace.doc_mut(id) else {
            continue;
        };
        let before = (doc.disk_mark(), doc.readonly());
        if let Some(reaction) = doc.check_disk() {
            events.push((doc.name.clone(), reaction));
        }
        changed |= (doc.disk_mark(), doc.readonly()) != before;
    }
    if let Some(text) = disk_message::summary(&events) {
        app.ui.set_message(text);
        changed = true;
    }
    changed
}
