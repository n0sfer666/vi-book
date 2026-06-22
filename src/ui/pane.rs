use crate::markdown::rendered::Rendered;
use crate::markdown::wrap_mode::WrapMode;
use crate::ui::app::App;
use crate::ui::view::View;
use crate::vim::editor::Editor;

const MAX_PANES: usize = 3;

pub struct Pane {
    pub editor: Editor,
    pub path: String,
    pub saved_text: String,
    pub top: usize,
    pub view: View,
    pub read_cursor: (usize, usize),
    pub read_pending_g: bool,
    pub read_cache: Option<(String, usize, WrapMode, Rendered)>,
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

    fn make_pane(&self, target: Option<&str>) -> Pane {
        if let Some(name) = target {
            let rel = normalize(name);
            if let Ok(mut editor) = Editor::open(self.session.absolute(&rel)) {
                editor.set_insert_escape(self.insert_escape.clone());
                let saved_text = editor.text();
                return Pane {
                    editor,
                    path: rel,
                    saved_text,
                    top: 0,
                    view: View::Read,
                    read_cursor: (0, 0),
                    read_pending_g: false,
                    read_cache: None,
                };
            }
        }
        self.duplicate_active()
    }

    fn duplicate_active(&self) -> Pane {
        let mut editor = if self.path.is_empty() {
            Editor::new(&self.editor.text())
        } else {
            Editor::open(self.session.absolute(&self.path))
                .unwrap_or_else(|_| Editor::new(&self.editor.text()))
        };
        editor.set_insert_escape(self.insert_escape.clone());
        let saved_text = editor.text();
        Pane {
            editor,
            path: self.path.clone(),
            saved_text,
            top: 0,
            view: View::Read,
            read_cursor: (0, 0),
            read_pending_g: false,
            read_cache: None,
        }
    }

    pub(crate) fn take_active(&mut self) -> Pane {
        Pane {
            editor: std::mem::replace(&mut self.editor, Editor::new("")),
            path: std::mem::take(&mut self.path),
            saved_text: std::mem::take(&mut self.saved_text),
            top: self.top,
            view: self.view,
            read_cursor: self.read_cursor,
            read_pending_g: self.read_pending_g,
            read_cache: self.read_cache.take(),
        }
    }

    pub(crate) fn put_active(&mut self, pane: Pane) {
        self.editor = pane.editor;
        self.path = pane.path;
        self.saved_text = pane.saved_text;
        self.top = pane.top;
        self.view = pane.view;
        self.read_cursor = pane.read_cursor;
        self.read_pending_g = pane.read_pending_g;
        self.read_cache = pane.read_cache;
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
