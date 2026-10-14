use crate::app::App;
use crate::buffer_list::{BufferEntry, BufferList};
use crate::requests;
use crate::view::View;
use v_ide_editor::document::Document;
use v_ide_editor::module::Event;

impl App {
    pub fn load(&mut self, rel: &str) {
        match Document::open(self.workspace.absolute(rel)) {
            Ok(mut doc) => {
                doc.editor.set_insert_escape(self.insert_escape.clone());
                doc.name = rel.to_string();
                if let Some(id) = self.workspace.open(doc) {
                    let state = std::mem::take(&mut self.ui.read);
                    self.read_states.insert(id, state);
                }
                self.ui.read.top = 0;
                self.ui.left = 0;
                self.ui.read.view = View::Read;
                self.ui.read.read_cursor = (0, 0);
                self.ui.read.read_cache = None;
                self.ui.picker = None;
                self.ui.menu = None;
            }
            Err(err) => self.ui.message = format!("open failed: {err}"),
        }
    }

    pub fn open_rel(&mut self, rel: &str) {
        if self.workspace.current().name == rel {
            return;
        }
        if let Some(idx) = self.workspace.pool().iter().position(|b| b.doc.name == rel) {
            self.switch_to_buffer(idx);
            self.note_opened();
            return;
        }
        self.load(rel);
        if self.workspace.current().name == rel {
            let id = self.workspace.current_id();
            self.history.visit(rel);
            self.note_opened();
            if let Some(doc) = self.workspace.doc_mut(id) {
                doc.mark_saved();
            }
        }
    }

    fn note_opened(&mut self) {
        if let Some(path) = self.workspace.current().path.clone() {
            requests::emit(self, Event::NoteOpen(path));
        }
    }

    pub fn named_buffers(&self) -> Vec<BufferEntry> {
        let current = self.workspace.current();
        let mut out = Vec::new();
        if !current.name.is_empty() {
            out.push(BufferEntry {
                idx: 0,
                path: current.name.clone(),
                dirty: current.is_dirty(),
                current: true,
            });
        }
        for (i, buf) in self.workspace.pool().iter().enumerate() {
            if !buf.doc.name.is_empty() {
                out.push(BufferEntry {
                    idx: i + 1,
                    path: buf.doc.name.clone(),
                    dirty: buf.doc.is_dirty(),
                    current: false,
                });
            }
        }
        out
    }

    pub fn open_buffer_list(&mut self) {
        self.ui.buffer_list = Some(BufferList::new(self.named_buffers()));
    }

    pub fn switch_to_buffer(&mut self, idx: usize) {
        let Some(id) = self.workspace.switch(idx) else {
            return;
        };
        let state = std::mem::take(&mut self.ui.read);
        self.read_states.insert(id, state);
        let state = self
            .read_states
            .remove(&self.workspace.current_id())
            .unwrap_or_default();
        self.ui.read = state;
        self.ui.left = 0;
        let path = self.workspace.current().name.clone();
        self.history.visit(&path);
    }

    pub fn close_buffer(&mut self, idx: usize) {
        if self.is_buffer_dirty(idx) {
            self.ui.message = String::from("unsaved changes (write with :w or force-close)");
            return;
        }
        if idx == 0 {
            if self.workspace.pool().is_empty() {
                self.blank_active();
                self.open_picker();
            } else {
                self.switch_to_buffer(0);
                self.drop_buffer(0);
            }
        } else {
            self.drop_buffer(idx - 1);
        }
    }

    pub fn any_dirty(&self) -> bool {
        self.workspace
            .windows()
            .iter()
            .any(|w| w.buf.doc.is_dirty())
            || self.workspace.pool().iter().any(|b| b.doc.is_dirty())
    }

    fn is_buffer_dirty(&self, idx: usize) -> bool {
        if idx == 0 {
            self.workspace.current().is_dirty()
        } else {
            self.workspace
                .pool()
                .get(idx - 1)
                .is_some_and(|b| b.doc.is_dirty())
        }
    }

    fn drop_buffer(&mut self, idx: usize) {
        if let Some(buf) = self.workspace.remove(idx) {
            self.read_states.remove(&buf.id);
        }
    }

    fn blank_active(&mut self) {
        let _ = self.workspace.replace(Document::new(""));
        self.ui.read.top = 0;
        self.ui.left = 0;
        self.ui.read.read_cursor = (0, 0);
        self.ui.read.read_cache = None;
    }
}
