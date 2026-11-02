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
            self.ui.set_message(format!("max {MAX_PANES} columns"));
            return;
        }
        let buf = self.make_buffer(target);
        self.ui.read.left = self.ui.left;
        let same = buf.id == self.workspace.current_id();
        let view = self.ui.read.fork();
        self.stash_window();
        let _ = self.workspace.split(buf);
        self.ui.read = if same { view } else { ReadState::new() };
        if !same {
            self.ui.read.view = self.initial_view();
        }
        self.ui.left = self.ui.read.left;
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
            self.ui.set_message(String::from(UNSAVED));
            return;
        }
        for window in self.workspace.only() {
            self.window_states.remove(&window.id);
            self.read_states.retain(|(id, _), _| *id != window.id);
        }
    }

    pub fn close_active(&mut self) {
        if self.pane_count() > 1 && self.workspace.current().is_dirty() {
            self.ui.set_message(String::from(UNSAVED));
            return;
        }
        if let Some(window) = self.workspace.close_window() {
            self.window_states.remove(&window.id);
            self.read_states.retain(|(id, _), _| *id != window.id);
            self.restore_window();
        }
    }

    fn stash_window(&mut self) {
        self.ui.read.left = self.ui.left;
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
        self.ui.left = self.ui.read.left;
    }

    fn make_buffer(&mut self, target: Option<&str>) -> Buffer {
        if let Some(name) = target {
            let rel = normalize(name);
            if let Some(id) = self.workspace.find_path(&self.workspace.absolute(&rel)) {
                return Buffer { id };
            }
            if let Ok(mut doc) = Document::open(self.workspace.absolute(&rel)) {
                doc.name = rel;
                return self.fresh_buffer(doc);
            }
        }
        self.workspace.current_buffer()
    }

    fn fresh_buffer(&mut self, doc: Document) -> Buffer {
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
