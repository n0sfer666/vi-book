use crate::ui::app::App;
use crate::ui::buffer_list::{BufferEntry, BufferList};
use crate::ui::plugin;
use crate::ui::view::View;
use crate::vim::editor::Editor;

impl App {
    pub fn load(&mut self, rel: &str) {
        match Editor::open(self.session.absolute(rel)) {
            Ok(mut editor) => {
                editor.set_insert_escape(self.insert_escape.clone());
                let old = self.take_active();
                if !old.path.is_empty() {
                    self.buffers.insert(0, old);
                }
                self.saved_text = editor.text();
                self.editor = editor;
                self.path = rel.to_string();
                self.top = 0;
                self.left = 0;
                self.view = View::Read;
                self.read_cursor = (0, 0);
                self.read_cache = None;
                self.picker = None;
                self.menu = None;
            }
            Err(err) => self.message = format!("open failed: {err}"),
        }
    }

    pub fn open_rel(&mut self, rel: &str) {
        if self.path == rel {
            return;
        }
        if let Some(idx) = self.buffers.iter().position(|p| p.path == rel) {
            self.switch_to_buffer(idx);
            plugin::fire(self, "note_open");
            return;
        }
        self.load(rel);
        if self.path == rel {
            self.history.visit(rel);
            plugin::fire(self, "note_open");
            self.saved_text = self.editor.text();
        }
    }

    pub fn named_buffers(&self) -> Vec<BufferEntry> {
        let mut out = Vec::new();
        if !self.path.is_empty() {
            out.push(BufferEntry {
                idx: 0,
                path: self.path.clone(),
                dirty: self.editor.text() != self.saved_text,
                current: true,
            });
        }
        for (i, pane) in self.buffers.iter().enumerate() {
            if !pane.path.is_empty() {
                out.push(BufferEntry {
                    idx: i + 1,
                    path: pane.path.clone(),
                    dirty: pane.editor.text() != pane.saved_text,
                    current: false,
                });
            }
        }
        out
    }

    pub fn open_buffer_list(&mut self) {
        self.buffer_list = Some(BufferList::new(self.named_buffers()));
    }

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

    pub fn any_dirty(&self) -> bool {
        self.editor.text() != self.saved_text
            || self.buffers.iter().any(|p| p.editor.text() != p.saved_text)
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
