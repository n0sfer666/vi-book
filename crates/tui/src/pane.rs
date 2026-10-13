use crate::app::App;
use crate::read_state::ReadState;
use v_ide_editor::document::Document;
use v_ide_editor::workspace::Buffer;

const MAX_PANES: usize = 3;

pub struct Pane {
    pub buf: Buffer,
    pub state: ReadState,
}

impl App {
    pub fn pane_count(&self) -> usize {
        self.others.len() + 1
    }

    pub fn other_for_display(&self, display: usize) -> &Pane {
        let idx = if display < self.active {
            display
        } else {
            display - 1
        };
        &self.others[idx]
    }

    pub fn vsplit(&mut self, target: Option<&str>) {
        if self.pane_count() >= MAX_PANES {
            self.message = format!("max {MAX_PANES} columns");
            return;
        }
        let new_pane = self.make_pane(target);
        let current = self.take_active();
        self.others.insert(self.active, current);
        self.put_active(new_pane);
        self.active += 1;
    }

    pub fn focus_next(&mut self) {
        let count = self.pane_count();
        if count <= 1 {
            return;
        }
        let current = self.take_active();
        self.others.insert(self.active, current);
        let next = (self.active + 1) % count;
        let pane = self.others.remove(next);
        self.put_active(pane);
        self.active = next;
    }

    pub fn only(&mut self) {
        self.others.clear();
        self.active = 0;
    }

    pub fn close_active(&mut self) {
        if self.others.is_empty() {
            return;
        }
        let idx = self.active.min(self.others.len() - 1);
        let pane = self.others.remove(idx);
        self.put_active(pane);
        self.active = idx.min(self.others.len());
    }

    fn make_pane(&mut self, target: Option<&str>) -> Pane {
        if let Some(name) = target {
            let rel = normalize(name);
            if let Ok(mut doc) = Document::open(self.session.absolute(&rel)) {
                doc.name = rel;
                return self.fresh_pane(doc);
            }
        }
        self.duplicate_active()
    }

    fn duplicate_active(&mut self) -> Pane {
        let current = self.workspace.current();
        let text = current.editor.text();
        let name = current.name.clone();
        let mut doc = if name.is_empty() {
            Document::new(&text)
        } else {
            Document::open(self.session.absolute(&name)).unwrap_or_else(|_| Document::new(&text))
        };
        doc.name = name;
        self.fresh_pane(doc)
    }

    fn fresh_pane(&mut self, mut doc: Document) -> Pane {
        doc.editor.set_insert_escape(self.insert_escape.clone());
        Pane {
            buf: self.workspace.detach(doc),
            state: ReadState::new(),
        }
    }

    pub(crate) fn take_active(&mut self) -> Pane {
        Pane {
            buf: self.workspace.replace(Document::new("")),
            state: self.take_read_state(),
        }
    }

    pub(crate) fn put_active(&mut self, pane: Pane) {
        let _ = self.workspace.put_current(pane.buf);
        self.put_read_state(pane.state);
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
