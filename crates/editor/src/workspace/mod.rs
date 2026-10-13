mod buffer;
mod doc_id;

pub use buffer::Buffer;
pub use doc_id::DocId;

use crate::document::Document;

pub struct Workspace {
    current: Buffer,
    pool: Vec<Buffer>,
    next: u64,
}

impl Workspace {
    pub fn new(doc: Document) -> Self {
        Self {
            current: Buffer { id: DocId(0), doc },
            pool: Vec::new(),
            next: 1,
        }
    }

    pub fn current(&self) -> &Document {
        &self.current.doc
    }

    pub fn current_mut(&mut self) -> &mut Document {
        &mut self.current.doc
    }

    pub fn current_id(&self) -> DocId {
        self.current.id
    }

    pub fn pool(&self) -> &[Buffer] {
        &self.pool
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
        std::mem::replace(&mut self.current, buf)
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
}
