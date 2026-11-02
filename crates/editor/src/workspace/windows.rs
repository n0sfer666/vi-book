use super::{Buffer, Window, WindowId, Workspace};
use std::collections::HashMap;

impl Workspace {
    pub fn split(&mut self, buffer: Buffer) -> Option<WindowId> {
        if !self.documents.contains_key(&buffer.id) {
            return None;
        }
        self.save_view();
        let source = &self.windows[self.active];
        let mut options = HashMap::new();
        if buffer.id == source.buf.id {
            options.insert(buffer.id, source.options[&buffer.id].clone());
        }
        let defaults = source.defaults.clone();
        let id = WindowId(self.next_window);
        self.next_window += 1;
        let mut cursors = HashMap::new();
        if let Some(doc) = self.documents.get_mut(&buffer.id) {
            cursors.insert(buffer.id, doc.editor.cursor_state());
        }
        self.active += 1;
        self.windows.insert(
            self.active,
            Window {
                id,
                buf: buffer,
                cursors,
                options,
                defaults,
            },
        );
        self.pool.retain(|buf| buf.id != buffer.id);
        self.restore_view();
        Some(id)
    }

    pub fn focus_next(&mut self) {
        self.save_view();
        self.active = (self.active + 1) % self.windows.len();
        self.restore_view();
    }

    #[must_use]
    pub fn close_window(&mut self) -> Option<Window> {
        if self.windows.len() <= 1 {
            return None;
        }
        self.save_view();
        let closed = self.windows.remove(self.active);
        self.active = self.active.min(self.windows.len() - 1);
        self.park(closed.buf);
        self.restore_view();
        Some(closed)
    }

    #[must_use]
    pub fn only(&mut self) -> Vec<Window> {
        self.save_view();
        let kept = self.windows.remove(self.active);
        self.active = 0;
        let closed = std::mem::replace(&mut self.windows, vec![kept]);
        for window in &closed {
            if let Some(options) = window.options.get(&window.buf.id) {
                self.last_options.insert(window.buf.id, options.clone());
            }
        }
        for window in closed.iter().rev() {
            self.park(window.buf);
        }
        self.restore_view();
        closed
    }
}
