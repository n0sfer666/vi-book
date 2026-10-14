mod buffer;
mod doc_id;
mod window;
mod window_id;

pub use buffer::Buffer;
pub use doc_id::DocId;
pub use window::Window;
pub use window_id::WindowId;

use std::path::{Path, PathBuf};

use crate::document::Document;

pub struct Workspace {
    windows: Vec<Window>,
    active: usize,
    pool: Vec<Buffer>,
    next: u64,
    next_window: u64,
    cwd: PathBuf,
}

impl Workspace {
    pub fn new(doc: Document) -> Self {
        Self {
            windows: vec![Window {
                id: WindowId(0),
                buf: Buffer { id: DocId(0), doc },
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
        self.cwd.join(rel)
    }

    pub fn current(&self) -> &Document {
        &self.windows[self.active].buf.doc
    }

    pub fn current_mut(&mut self) -> &mut Document {
        &mut self.windows[self.active].buf.doc
    }

    pub fn current_id(&self) -> DocId {
        self.windows[self.active].buf.id
    }

    pub fn pool(&self) -> &[Buffer] {
        &self.pool
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn active_window(&self) -> WindowId {
        self.windows[self.active].id
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    #[must_use]
    pub fn detach(&mut self, doc: Document) -> Buffer {
        let id = DocId(self.next);
        self.next += 1;
        Buffer { id, doc }
    }

    pub fn add(&mut self, doc: Document) -> DocId {
        let buf = self.detach(doc);
        let id = buf.id;
        self.pool.push(buf);
        id
    }

    #[must_use]
    pub fn replace(&mut self, doc: Document) -> Buffer {
        let buf = self.detach(doc);
        self.put_current(buf)
    }

    #[must_use]
    pub fn put_current(&mut self, buf: Buffer) -> Buffer {
        std::mem::replace(&mut self.windows[self.active].buf, buf)
    }

    pub fn open(&mut self, doc: Document) -> Option<DocId> {
        let old = self.replace(doc);
        if old.doc.name.is_empty() {
            return None;
        }
        let id = old.id;
        self.pool.insert(0, old);
        Some(id)
    }

    pub fn switch(&mut self, idx: usize) -> Option<DocId> {
        if idx >= self.pool.len() {
            return None;
        }
        let target = self.pool.remove(idx);
        let old = self.put_current(target);
        let id = old.id;
        self.pool.insert(0, old);
        Some(id)
    }

    #[must_use]
    pub fn remove(&mut self, idx: usize) -> Option<Buffer> {
        (idx < self.pool.len()).then(|| self.pool.remove(idx))
    }

    pub fn split(&mut self, buf: Buffer) -> WindowId {
        let id = WindowId(self.next_window);
        self.next_window += 1;
        self.active += 1;
        self.windows.insert(self.active, Window { id, buf });
        id
    }

    pub fn focus_next(&mut self) {
        self.active = (self.active + 1) % self.windows.len();
    }

    #[must_use]
    pub fn close_window(&mut self) -> Option<Window> {
        if self.windows.len() <= 1 {
            return None;
        }
        let closed = self.windows.remove(self.active);
        self.active = self.active.min(self.windows.len() - 1);
        Some(closed)
    }

    #[must_use]
    pub fn only(&mut self) -> Vec<Window> {
        let kept = self.windows.remove(self.active);
        self.active = 0;
        std::mem::replace(&mut self.windows, vec![kept])
    }
}
