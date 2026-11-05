use crate::app::App;
use crate::{core_exec, disk_message};
use v_ide_editor::{disk::Reaction, document::DiskCheck, execution::Error};

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
        let reaction = match doc.prepare_disk_check() {
            DiskCheck::Done(reaction) => reaction,
            DiskCheck::Reload(reload) => {
                let (result, effects) =
                    core_exec::with_buffer_context(app, id, |cx| cx.reload_disk(id, reload));
                changed = true;
                if effects.navigated {
                    break;
                }
                match result {
                    Some(Ok(reaction)) => reaction,
                    Some(Err(Error::Autocmd(_))) | None => None,
                    Some(Err(error)) => {
                        app.ui.set_message(format!("reload failed: {error}"));
                        None
                    }
                }
            }
        };
        if let Some(reaction) = reaction {
            changed |= reaction == Reaction::Reload;
            if let Some(doc) = app.workspace.doc(id) {
                events.push((doc.name.clone(), reaction));
            }
        }
        if let Some(doc) = app.workspace.doc(id) {
            changed |= (doc.disk_mark(), doc.readonly()) != before;
        }
    }
    if let Some(text) = disk_message::summary(&events) {
        app.ui.set_message(text);
        changed = true;
    }
    changed
}
