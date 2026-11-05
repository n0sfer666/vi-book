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
        self.open_rel_with_navigation(rel);
    }

    pub fn open_initial_rel(&mut self, rel: &str) -> bool {
        self.open_rel_with_navigation_mode(rel, true)
    }

    pub(crate) fn open_rel_with_navigation(&mut self, rel: &str) -> bool {
        self.open_rel_with_navigation_mode(rel, false)
    }

    fn open_rel_with_navigation_mode(&mut self, rel: &str, startup: bool) -> bool {
        let old = self.workspace.current_id();
        if let Some(id) = self.workspace.find_path(&self.workspace.absolute(rel)) {
            if id == old {
                return true;
            }
            let (switched, navigated) = self.switch_document(id);
            if switched {
                self.note_opened();
            }
            return navigated;
        }
        let (result, effects) =
            crate::core_exec::with_outcome(self, |cx| cx.open_existing_path(rel, startup));
        if let Err(error) = result {
            if !matches!(error, v_ide_editor::execution::Error::Autocmd(_)) {
                self.ui.set_message(format!("open failed: {error}"));
            }
            return effects.navigated || old != self.workspace.current_id();
        }
        let id = self.workspace.current_id();
        if self.workspace.current().name == rel {
            if startup && old == id {
                self.ui.read.view = self.initial_view();
            }
            self.ui.picker = None;
            self.ui.menu = None;
            #[cfg(feature = "notebook")]
            self.history.visit(rel);
            let already_dirty = self.workspace.current().is_dirty();
            self.note_opened();
            if !already_dirty && let Some(doc) = self.workspace.doc_mut(id) {
                doc.mark_saved();
            }
            return true;
        }
        effects.navigated || old != self.workspace.current_id()
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

    fn switch_document(&mut self, target: v_ide_editor::workspace::DocId) -> (bool, bool) {
        if target == self.workspace.current_id() || self.workspace.doc(target).is_none() {
            return (false, false);
        }
        let (result, effects) =
            crate::core_exec::with_outcome(self, |cx| cx.select_document(target));
        if let Err(error) = result {
            if !matches!(error, v_ide_editor::execution::Error::Autocmd(_)) {
                self.ui.set_message(error.to_string());
            }
            return (false, effects.navigated);
        }
        #[cfg(feature = "notebook")]
        self.history.visit(&self.workspace.current().name);
        (true, true)
    }

    pub fn close_buffer(&mut self, idx: usize) {
        let current = self.workspace.current_id();
        let pool = self.workspace.pool();
        let id = if idx == 0 {
            current
        } else if let Some(buffer) = pool.get(idx - 1) {
            buffer.id
        } else {
            return;
        };
        #[cfg(any(feature = "vault", feature = "notebook"))]
        let last = idx == 0 && pool.is_empty();
        let result = crate::core_exec::with_context(self, |cx| cx.close_document(id));
        if self.workspace.doc(id).is_none() {
            self.read_states.retain(|(_, doc), _| *doc != id);
        }
        if let Err(error) = &result
            && !matches!(error, v_ide_editor::execution::Error::Autocmd(_))
        {
            self.ui.set_message(error.to_string());
        }
        #[cfg(feature = "notebook")]
        if result.is_ok() && idx == 0 && !last && current != self.workspace.current_id() {
            self.history.visit(&self.workspace.current().name);
        }
        #[cfg(feature = "vault")]
        if result.is_ok() && last {
            self.open_picker();
        }
    }

    pub fn any_dirty(&self) -> bool {
        self.workspace.documents().any(|(_, doc)| doc.is_dirty())
    }
}
