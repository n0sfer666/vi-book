use std::collections::HashMap;

use crate::{app::App, core_exec, view::View};
use v_ide_editor::workspace::{DocId, WindowId, Workspace};
use v_ide_engine::mode::Mode;
use v_ide_keys::{Code, Key, Mods};
use v_ide_text::Position;

use super::current;

#[derive(Clone, Copy)]
struct Cursor {
    window: WindowId,
    doc: DocId,
    position: Position,
    tick: u64,
}

impl Cursor {
    fn current(workspace: &Workspace) -> Self {
        let editor = &workspace.current().editor;
        Self {
            window: workspace.active_window(),
            doc: workspace.current_id(),
            position: editor.cursor(),
            tick: editor.change_tick(),
        }
    }
}

pub(super) struct Input {
    cursor: Cursor,
    normal: HashMap<DocId, u64>,
    insert: Option<(DocId, u64)>,
}

impl Input {
    pub(super) fn new(workspace: &Workspace) -> Self {
        let cursor = Cursor::current(workspace);
        Self {
            cursor,
            normal: HashMap::from([(cursor.doc, cursor.tick)]),
            insert: None,
        }
    }

    fn seen(&self, doc: DocId, insert: bool) -> u64 {
        if insert
            && let Some((id, tick)) = self.insert
            && id == doc
        {
            return tick;
        }
        self.normal.get(&doc).copied().unwrap_or_default()
    }

    fn observe(&mut self, doc: DocId, tick: u64, insert: bool) {
        if insert {
            self.insert = Some((doc, tick));
        } else {
            self.normal.insert(doc, tick);
        }
    }

    fn leave_insert(&mut self, doc: DocId, escaped: bool) {
        if let Some((id, tick)) = self.insert.take()
            && id == doc
            && escaped
        {
            self.normal.insert(id, tick);
        }
    }
}

pub(crate) fn editor_key(app: &mut App, key: Key) {
    let doc = app.workspace.current_id();
    let before = app.workspace.current().editor.mode();
    let escaping = before == Mode::Insert && key == Key::new(Code::Esc, Mods::NONE);
    if escaping {
        current(app, "InsertLeavePre");
        if let Some(doc) = app.workspace.doc_mut(doc) {
            doc.editor.defer_insert_undo();
        }
    }
    if escaping && app.workspace.current_id() != doc {
        if let Some(doc) = app.workspace.doc_mut(doc) {
            doc.editor.handle_key(key);
        }
    } else {
        app.workspace.current_mut().editor.handle_key(key);
    }

    let after = app.workspace.doc(doc).map(|doc| doc.editor.mode());
    if before != Mode::Insert && after == Some(Mode::Insert) && app.workspace.current_id() == doc {
        insert_enter(app, doc);
    }
    if before == Mode::Insert && after != Some(Mode::Insert) {
        app.autocmd.input.leave_insert(doc, escaping);
        if escaping {
            current(app, "InsertLeave");
            if let Some(doc) = app.workspace.doc_mut(doc) {
                doc.editor.commit_insert_undo();
            }
        }
    }
}

fn insert_enter(app: &mut App, doc: DocId) {
    app.workspace.current_mut().editor.pause_insert_entry();
    current(app, "InsertEnter");
    resume_insert(app, doc);
}

fn resume_insert(app: &mut App, doc: DocId) {
    let active = app.workspace.current_id();
    if let Some(document) = app.workspace.doc_mut(doc) {
        document.editor.finish_insert_entry(active == doc);
    }
    if active != doc {
        app.ui.read.view = View::Edit;
        app.ui.read_select = None;
        app.workspace.current_mut().editor.start_insert_here();
    }
    let seen = app.autocmd.input.seen(active, false);
    app.autocmd.input.insert = Some((active, seen));
}

pub(crate) fn flush(app: &mut App) -> bool {
    if app.autocmd.booting || app.quit || app.ui.command.is_some() || app.ui.pager.is_some() {
        return false;
    }
    let cursor = Cursor::current(&app.workspace);
    let last = app.autocmd.input.cursor;
    let moved = last.window != cursor.window
        || last.position != cursor.position
        || (last.doc == cursor.doc
            && last.tick != cursor.tick
            && cursor.tick > app.workspace.current().read_tick());
    if moved {
        let inserting = app.workspace.current().editor.mode() == Mode::Insert;
        current(
            app,
            if inserting {
                "CursorMovedI"
            } else {
                "CursorMoved"
            },
        );
        if inserting && app.workspace.current_id() != cursor.doc {
            resume_insert(app, cursor.doc);
        }
    }

    let doc = app.workspace.current();
    let id = app.workspace.current_id();
    let inserting = doc.editor.mode() == Mode::Insert;
    let baseline = app.autocmd.input.seen(id, inserting).max(doc.read_tick());
    let changed = doc.editor.change_tick() > baseline;
    if changed {
        if inserting {
            let _ = core_exec::with_context(app, |cx| cx.emit_insert_change(id));
            if app.workspace.current_id() != id {
                resume_insert(app, id);
            }
        } else {
            current(app, "TextChanged");
        }
        let active = app.workspace.current_id();
        let doc = app.workspace.current();
        app.autocmd.input.observe(
            active,
            doc.editor.change_tick(),
            doc.editor.mode() == Mode::Insert,
        );
    }
    app.autocmd.input.cursor = Cursor::current(&app.workspace);
    if moved || changed || last.doc != app.workspace.current_id() {
        app.autocmd
            .input
            .normal
            .retain(|id, _| app.workspace.doc(*id).is_some());
    }
    moved || changed
}
