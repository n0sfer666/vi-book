use crate::ui::app::App;
use crate::vim::editor::Editor;

impl App {
    pub fn switch_to_buffer(&mut self, idx: usize) {
        if idx >= self.buffers.len() {
            return;
        }
        let old = self.take_active();
        self.buffers.insert(0, old);
        let target = self.buffers.remove(idx + 1);
        let path = target.path.clone();
        self.put_active(target);
        self.left = 0;
        self.history.visit(&path);
    }

    pub fn close_buffer(&mut self, idx: usize) {
        if self.is_buffer_dirty(idx) {
            self.message = String::from("unsaved changes (write with :w or force-close)");
            return;
        }
        if idx == 0 {
            if self.buffers.is_empty() {
                self.blank_active();
                self.open_picker();
            } else {
                self.switch_to_buffer(0);
                self.buffers.remove(0);
            }
        } else if idx - 1 < self.buffers.len() {
            self.buffers.remove(idx - 1);
        }
    }

    fn is_buffer_dirty(&self, idx: usize) -> bool {
        if idx == 0 {
            self.editor.text() != self.saved_text
        } else {
            self.buffers
                .get(idx - 1)
                .is_some_and(|p| p.editor.text() != p.saved_text)
        }
    }

    fn blank_active(&mut self) {
        self.editor = Editor::new("");
        self.path = String::new();
        self.saved_text = String::new();
        self.top = 0;
        self.left = 0;
        self.read_cursor = (0, 0);
        self.read_cache = None;
    }
}
