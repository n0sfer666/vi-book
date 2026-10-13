use crate::app::App;
use crate::read_state::ReadState;
use v_ide_editor::document::Document;
use v_ide_editor::workspace::Buffer;

const MAX_PANES: usize = 3;
const UNSAVED: &str = "unsaved changes (write with :w first)";

impl App {
    pub fn pane_count(&self) -> usize {
        self.workspace.windows().len()
    }

    pub fn vsplit(&mut self, target: Option<&str>) {
        if self.pane_count() >= MAX_PANES {
            self.ui.message = format!("max {MAX_PANES} columns");
            return;
        }
        let buf = self.make_buffer(target);
        self.stash_window();
        self.workspace.split(buf);
        self.ui.read = ReadState::new();
    }

    pub fn focus_next(&mut self) {
        self.stash_window();
        self.workspace.focus_next();
        self.restore_window();
    }

    pub fn only(&mut self) {
        let active = self.workspace.active_window();
        let others_dirty = self
            .workspace
            .windows()
            .iter()
            .any(|w| w.id != active && w.buf.doc.is_dirty());
        if others_dirty {
            self.ui.message = String::from(UNSAVED);
            return;
        }
        for window in self.workspace.only() {
            self.window_states.remove(&window.id);
        }
    }

    pub fn close_active(&mut self) {
        if self.pane_count() > 1 && self.workspace.current().is_dirty() {
            self.ui.message = String::from(UNSAVED);
            return;
        }
        if self.workspace.close_window().is_some() {
            self.restore_window();
        }
    }

    fn stash_window(&mut self) {
        let state = std::mem::take(&mut self.ui.read);
        self.window_states
            .insert(self.workspace.active_window(), state);
    }

    fn restore_window(&mut self) {
        let state = self
            .window_states
            .remove(&self.workspace.active_window())
            .unwrap_or_default();
        self.ui.read = state;
    }

    fn make_buffer(&mut self, target: Option<&str>) -> Buffer {
        if let Some(name) = target {
            let rel = normalize(name);
            if let Ok(mut doc) = Document::open(self.session.absolute(&rel)) {
                doc.name = rel;
                return self.fresh_buffer(doc);
            }
        }
        self.duplicate_active()
    }

    fn duplicate_active(&mut self) -> Buffer {
        let current = self.workspace.current();
        let text = current.editor.text();
        let name = current.name.clone();
        let mut doc = if name.is_empty() {
            Document::new(&text)
        } else {
            Document::open(self.session.absolute(&name)).unwrap_or_else(|_| Document::new(&text))
        };
        doc.name = name;
        self.fresh_buffer(doc)
    }

    fn fresh_buffer(&mut self, mut doc: Document) -> Buffer {
        doc.editor.set_insert_escape(self.insert_escape.clone());
        self.workspace.detach(doc)
    }
}

fn normalize(name: &str) -> String {
    let name = name.trim();
    if name.contains('.') {
        name.to_string()
    } else {
        format!("{name}.md")
    }
}
