use crate::core::buffer::Buffer;

#[derive(Debug, Clone)]
pub struct Edit {
    pub at: usize,
    pub removed: String,
    pub inserted: String,
}

impl Edit {
    pub fn insertion(at: usize, text: String) -> Self {
        Self {
            at,
            removed: String::new(),
            inserted: text,
        }
    }

    pub fn removal(at: usize, text: String) -> Self {
        Self {
            at,
            removed: text,
            inserted: String::new(),
        }
    }

    pub fn apply(&self, buffer: &mut Buffer) {
        let removed_len = self.removed.chars().count();
        if removed_len > 0 {
            buffer.remove(self.at..self.at + removed_len);
        }
        if !self.inserted.is_empty() {
            buffer.insert(self.at, &self.inserted);
        }
    }

    pub fn inverted(&self) -> Edit {
        Edit {
            at: self.at,
            removed: self.inserted.clone(),
            inserted: self.removed.clone(),
        }
    }
}
