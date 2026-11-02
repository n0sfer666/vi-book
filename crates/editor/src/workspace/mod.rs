mod buffer;
mod buffers;
mod doc_id;
mod identity;
mod options;
mod views;
mod window;
mod window_id;
mod windows;

pub use buffer::{Buffer, BufferRef};
pub use doc_id::DocId;
pub use window::{Window, WindowRef};
pub use window_id::WindowId;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::document::Document;
use crate::options::{Scope, Values};

pub struct Workspace {
    documents: HashMap<DocId, Document>,
    options: Values,
    last_options: HashMap<DocId, Values>,
    windows: Vec<Window>,
    active: usize,
    pool: Vec<Buffer>,
    next: u64,
    next_window: u64,
    cwd: PathBuf,
}

impl Workspace {
    pub fn new(mut doc: Document) -> Self {
        let id = DocId(0);
        let mut cursors = HashMap::new();
        cursors.insert(id, doc.editor.cursor_state());
        let options = Values::default();
        let window_options = HashMap::from([(id, options.scoped(Scope::Window))]);
        Self {
            last_options: HashMap::new(),
            options: options.clone(),
            documents: HashMap::from([(id, doc)]),
            windows: vec![Window {
                id: WindowId(0),
                buf: Buffer { id },
                cursors,
                options: window_options,
                defaults: options.scoped(Scope::Window),
            }],
            active: 0,
            pool: Vec::new(),
            next: 1,
            next_window: 1,
            cwd: PathBuf::from("."),
        }
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }
    pub fn set_cwd(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
    }
    pub fn absolute(&self, rel: &str) -> PathBuf {
        identity::absolute(self.cwd.join(rel))
    }

    pub fn current(&self) -> &Document {
        &self.documents[&self.current_id()]
    }
    pub fn current_mut(&mut self) -> &mut Document {
        let id = self.current_id();
        self.documents
            .get_mut(&id)
            .expect("active window references a registered document")
    }
    pub fn current_id(&self) -> DocId {
        self.windows[self.active].buf.id
    }
    pub fn current_buffer(&self) -> Buffer {
        self.windows[self.active].buf
    }
    pub fn doc(&self, id: DocId) -> Option<&Document> {
        self.documents.get(&id)
    }
    pub fn doc_mut(&mut self, id: DocId) -> Option<&mut Document> {
        self.documents.get_mut(&id)
    }
    pub fn documents(&self) -> impl Iterator<Item = (DocId, &Document)> {
        self.documents.iter().map(|(&id, doc)| (id, doc))
    }
    pub fn ids(&self) -> Vec<DocId> {
        let mut ids = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for id in self
            .windows
            .iter()
            .map(|window| window.buf.id)
            .chain(self.pool.iter().map(|buf| buf.id))
        {
            if seen.insert(id) {
                ids.push(id);
            }
        }
        let mut remaining: Vec<_> = self
            .documents
            .keys()
            .copied()
            .filter(|id| !seen.contains(id))
            .collect();
        remaining.sort_by_key(|id| id.0);
        ids.extend(remaining);
        ids
    }

    pub fn pool(&self) -> Vec<BufferRef<'_>> {
        self.pool
            .iter()
            .map(|buf| BufferRef {
                id: buf.id,
                doc: &self.documents[&buf.id],
            })
            .collect()
    }
    pub fn windows(&self) -> Vec<WindowRef<'_>> {
        self.windows
            .iter()
            .map(|window| {
                let doc = &self.documents[&window.buf.id];
                let cursor = if window.id == self.active_window() {
                    doc.editor.cursor()
                } else {
                    window
                        .cursors
                        .get(&window.buf.id)
                        .map_or_else(|| doc.editor.cursor(), |state| state.position())
                };
                WindowRef {
                    id: window.id,
                    buf: BufferRef {
                        id: window.buf.id,
                        doc,
                    },
                    cursor,
                    options: &window.options[&window.buf.id],
                }
            })
            .collect()
    }
    pub fn active_window(&self) -> WindowId {
        self.windows[self.active].id
    }
    pub fn active_index(&self) -> usize {
        self.active
    }

    fn visible(&self, id: DocId) -> bool {
        self.windows.iter().any(|window| window.buf.id == id)
    }
    fn park(&mut self, buffer: Buffer) {
        if !self.visible(buffer.id) && !self.pool.iter().any(|b| b.id == buffer.id) {
            self.pool.insert(0, buffer);
        }
    }
}
