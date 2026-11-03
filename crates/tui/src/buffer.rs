use crate::app::App;
use crate::buffer_list::{BufferEntry, BufferList};
use crate::requests;
use v_ide_editor::document::Document;
use v_ide_editor::module::Event;

impl App {
    pub fn load(&mut self, rel: &str) {
        match Document::open(self.workspace.absolute(rel)) {
            Ok(mut doc) => {
                doc.name = rel.to_string();
                self.ui.read.left = self.ui.left;
                if let Some(id) = self.workspace.open(doc) {
                    let state = std::mem::take(&mut self.ui.read);
                    self.read_states
                        .insert((self.workspace.active_window(), id), state);
                }
                self.ui.read.top = 0;
                self.ui.left = 0;
                self.ui.read.view = self.initial_view();
                self.ui.read.read_cursor = (0, 0);
                self.ui.read.read_cache = None;
                self.ui.picker = None;
                self.ui.menu = None;
            }
            Err(err) => self.ui.set_message(format!("open failed: {err}")),
        }
    }

    pub fn open_rel(&mut self, rel: &str) {
        if let Some(id) = self.workspace.find_path(&self.workspace.absolute(rel)) {
            if id == self.workspace.current_id() {
                return;
            }
            if self.switch_document(id) {
                self.note_opened();
            }
            return;
        }
        self.load(rel);
        if self.workspace.current().name == rel {
            let id = self.workspace.current_id();
            #[cfg(feature = "notebook")]
            self.history.visit(rel);
            self.note_opened();
            if let Some(doc) = self.workspace.doc_mut(id) {
                doc.mark_saved();
            }
        }
    }

    fn note_opened(&mut self) {
        if let Some(filetype) = self.workspace.current().filetype() {
            let activated = self.modules.filetype(&self.workspace, filetype);
            requests::apply(self, activated);
        }
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
                mark: current.disk_mark(),
            });
        }
        for (i, buf) in self.workspace.pool().iter().enumerate() {
            if !buf.doc.name.is_empty() {
                out.push(BufferEntry {
                    idx: i + 1,
                    path: buf.doc.name.clone(),
                    dirty: buf.doc.is_dirty(),
                    current: false,
                    mark: buf.doc.disk_mark(),
                });
            }
        }
        out
    }

    pub fn open_buffer_list(&mut self) {
        self.ui.buffer_list = Some(BufferList::new(self.named_buffers()));
    }

    pub fn switch_to_buffer(&mut self, idx: usize) {
        let target = self.workspace.pool().get(idx).map(|buffer| buffer.id);
        if let Some(id) = target {
            self.switch_document(id);
        }
    }

    fn switch_document(&mut self, target: v_ide_editor::workspace::DocId) -> bool {
        if target == self.workspace.current_id() || self.workspace.doc(target).is_none() {
            return false;
        }
        let result = crate::core_exec::with_context(self, |cx| cx.select_document(target));
        if let Err(error) = result {
            if !matches!(error, v_ide_editor::execution::Error::Autocmd(_)) {
                self.ui.set_message(error.to_string());
            }
            return false;
        }
        #[cfg(feature = "notebook")]
        self.history.visit(&self.workspace.current().name);
        true
    }

    pub fn close_buffer(&mut self, idx: usize) {
        if self.is_buffer_dirty(idx) {
            self.ui.set_message(String::from(
                "unsaved changes (write with :w or force-close)",
            ));
            return;
        }
        if idx == 0 {
            if self.workspace.pool().is_empty() {
                self.blank_active();
                #[cfg(feature = "vault")]
                self.open_picker();
            } else {
                let closing = self.workspace.current_id();
                self.switch_to_buffer(0);
                let index = self
                    .workspace
                    .pool()
                    .iter()
                    .position(|buffer| buffer.id == closing);
                if let Some(index) = index {
                    self.drop_buffer(index);
                }
            }
        } else {
            self.drop_buffer(idx - 1);
        }
    }

    pub fn any_dirty(&self) -> bool {
        self.workspace.documents().any(|(_, doc)| doc.is_dirty())
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
        if self.is_buffer_dirty(idx + 1) {
            self.ui.set_message(String::from(
                "unsaved changes (write with :w or force-close)",
            ));
            return;
        }
        if let Some(buf) = self.workspace.remove(idx) {
            self.read_states.retain(|(_, id), _| *id != buf.id);
        }
    }

    fn blank_active(&mut self) {
        let old = self.workspace.replace(Document::new(""));
        let _ = self.workspace.forget(old.id);
        self.ui.read.top = 0;
        self.ui.left = 0;
        self.ui.read.read_cursor = (0, 0);
        self.ui.read.read_cache = None;
    }
}
