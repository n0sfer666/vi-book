use super::{Buffer, DocId, Workspace};
use crate::document::Document;

impl Workspace {
    #[must_use]
    pub fn detach(&mut self, doc: Document) -> Buffer {
        if let Some(id) = doc
            .path
            .as_ref()
            .and_then(|path| self.find_path(&super::identity::absolute(path.to_path_buf())))
        {
            return Buffer { id };
        }
        let id = DocId(self.next);
        self.next += 1;
        self.documents.insert(id, doc);
        Buffer { id }
    }

    pub fn add(&mut self, doc: Document) -> DocId {
        let buffer = self.detach(doc);
        if !self.visible(buffer.id) && !self.pool.iter().any(|buf| buf.id == buffer.id) {
            self.pool.push(buffer);
        }
        buffer.id
    }

    #[must_use]
    pub fn replace(&mut self, doc: Document) -> Buffer {
        let buffer = self.detach(doc);
        self.activate(buffer)
    }

    #[must_use]
    pub fn put_current(&mut self, buffer: Buffer) -> Option<Buffer> {
        self.documents
            .contains_key(&buffer.id)
            .then(|| self.activate(buffer))
    }

    pub fn select(&mut self, id: DocId) -> Option<DocId> {
        if id == self.current_id() || !self.documents.contains_key(&id) {
            return None;
        }
        let old = self.activate(Buffer { id });
        self.keep_previous(old)
    }

    pub fn open(&mut self, doc: Document) -> Option<DocId> {
        let buffer = self.detach(doc);
        self.select(buffer.id)
    }

    fn keep_previous(&mut self, old: Buffer) -> Option<DocId> {
        let doc = &self.documents[&old.id];
        if doc.name.is_empty()
            && !doc.is_dirty()
            && doc.editor.text().is_empty()
            && !self.visible(old.id)
        {
            let _ = self.forget(old.id);
            None
        } else if self.visible(old.id) {
            Some(old.id)
        } else {
            self.park(old);
            Some(old.id)
        }
    }

    pub fn switch(&mut self, index: usize) -> Option<DocId> {
        let buffer = *self.pool.get(index)?;
        self.select(buffer.id)
    }

    #[must_use]
    pub fn remove(&mut self, index: usize) -> Option<Buffer> {
        if index >= self.pool.len() {
            return None;
        }
        let buffer = self.pool.remove(index);
        let _ = self.forget(buffer.id);
        Some(buffer)
    }

    pub fn forget(&mut self, id: DocId) -> Option<Document> {
        if self.visible(id) || self.pool.iter().any(|buffer| buffer.id == id) {
            return None;
        }
        for window in &mut self.windows {
            window.cursors.remove(&id);
        }
        self.documents.remove(&id)
    }
}
