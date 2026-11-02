use super::{Buffer, Workspace};
use crate::options::Scope;

impl Workspace {
    pub(super) fn save_view(&mut self) {
        let id = self.current_id();
        let state = self.current_mut().editor.cursor_state();
        self.windows[self.active].cursors.insert(id, state);
        if let Some(options) = self.windows[self.active].options.get(&id) {
            self.last_options.insert(id, options.clone());
        }
    }

    pub(super) fn restore_view(&mut self) {
        let doc = self.current_id();
        let window = &mut self.windows[self.active];
        window.options.entry(doc).or_insert_with(|| {
            self.last_options
                .get(&doc)
                .cloned()
                .unwrap_or_else(|| window.defaults.scoped(Scope::Window))
        });
        let window = &self.windows[self.active];
        if let Some(state) = window.cursors.get(&window.buf.id)
            && let Some(doc) = self.documents.get_mut(&window.buf.id)
        {
            doc.editor.restore_cursor_state(state);
        }
    }

    pub(super) fn activate(&mut self, buffer: Buffer) -> Buffer {
        self.save_view();
        let old = std::mem::replace(&mut self.windows[self.active].buf, buffer);
        self.restore_view();
        self.pool.retain(|buf| buf.id != buffer.id);
        old
    }
}
