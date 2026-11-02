use crate::buffer::Buffer;

#[derive(Debug, Clone)]
pub struct Edit {
    pub at: usize,
    pub removed: String,
    pub inserted: String,
    adjustment: Adjustment,
    undo: bool,
}

#[derive(Debug, Clone)]
enum Adjustment {
    Chars,
    Preserve,
    Lines {
        at: usize,
        removed: usize,
        inserted: usize,
    },
}

impl Edit {
    pub fn insertion(at: usize, text: String) -> Self {
        Self {
            at,
            removed: String::new(),
            inserted: text,
            adjustment: Adjustment::Chars,
            undo: false,
        }
    }

    pub fn removal(at: usize, text: String) -> Self {
        Self {
            at,
            removed: text,
            inserted: String::new(),
            adjustment: Adjustment::Chars,
            undo: false,
        }
    }

    pub fn apply(&self, buffer: &mut Buffer) {
        let removed_len = self.removed.chars().count();
        let empty = buffer.len_chars() == 0;
        let lines = match self.adjustment {
            Adjustment::Lines {
                at,
                removed,
                inserted,
            } => Some((at, removed, inserted)),
            Adjustment::Chars if self.undo => {
                let removed = self.removed.bytes().filter(|&byte| byte == b'\n').count();
                let inserted = self.inserted.bytes().filter(|&byte| byte == b'\n').count();
                let at =
                    buffer.char_to_pos(self.at).line + usize::from(removed == 0 && inserted > 0);
                (removed + inserted > 0).then_some((at, removed, inserted))
            }
            _ => None,
        };
        if let Some((at, removed, inserted)) = lines {
            buffer.replace_raw(self.at, removed_len, &self.inserted);
            if !(self.undo && empty) {
                buffer.adjust_lines(at, removed, inserted);
            }
            return;
        }
        if matches!(self.adjustment, Adjustment::Preserve) {
            buffer.replace_raw(self.at, removed_len, &self.inserted);
            return;
        }
        if removed_len > 0 {
            buffer.remove(self.at..self.at + removed_len);
        }
        if !self.inserted.is_empty() {
            buffer.insert(self.at, &self.inserted);
        }
    }

    pub fn inverted(&self) -> Edit {
        let adjustment = match self.adjustment {
            Adjustment::Lines {
                at,
                removed,
                inserted,
            } => Adjustment::Lines {
                at,
                removed: inserted,
                inserted: removed,
            },
            _ => self.adjustment.clone(),
        };
        Edit {
            at: self.at,
            removed: self.inserted.clone(),
            inserted: self.removed.clone(),
            adjustment,
            undo: !self.undo,
        }
    }

    pub fn replacement(at: usize, removed: String, inserted: String) -> Self {
        Self {
            at,
            removed,
            inserted,
            adjustment: Adjustment::Preserve,
            undo: false,
        }
    }

    pub fn line_insertion(at: usize, text: String, first: usize, count: usize) -> Self {
        Self {
            adjustment: Adjustment::Lines {
                at: first,
                removed: 0,
                inserted: count,
            },
            ..Self::insertion(at, text)
        }
    }

    pub fn line_removal(at: usize, text: String, first: usize, count: usize) -> Self {
        Self {
            adjustment: Adjustment::Lines {
                at: first,
                removed: count,
                inserted: 0,
            },
            ..Self::removal(at, text)
        }
    }
}
