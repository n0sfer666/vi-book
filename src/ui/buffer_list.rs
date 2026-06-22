use crate::ui::list_nav;
use crate::vault::fuzzy;

#[derive(Clone)]
pub struct BufferEntry {
    pub idx: usize,
    pub path: String,
    pub dirty: bool,
    pub current: bool,
}

impl BufferEntry {
    pub fn label(&self) -> String {
        let marker = if self.current { "% " } else { "  " };
        let dirty = if self.dirty { " [+]" } else { "" };
        format!("{marker}{}{dirty}", self.path)
    }
}

pub struct BufferList {
    entries: Vec<BufferEntry>,
    query: String,
    selected: usize,
}

impl BufferList {
    pub fn new(entries: Vec<BufferEntry>) -> Self {
        let selected = usize::from(entries.len() > 1);
        Self {
            entries,
            query: String::new(),
            selected,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn filtered(&self) -> Vec<&BufferEntry> {
        let mut scored: Vec<(i32, usize, &BufferEntry)> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| fuzzy::score(&self.query, &e.path).map(|s| (s, i, e)))
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        scored.into_iter().map(|(_, _, e)| e).collect()
    }

    pub fn current(&self) -> Option<&BufferEntry> {
        self.filtered().into_iter().nth(self.selected)
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn push(&mut self, c: char) {
        self.query.push(c);
        self.clamp();
    }

    pub fn pop(&mut self) {
        self.query.pop();
        self.clamp();
    }

    pub fn move_down(&mut self) {
        self.selected = list_nav::next(self.selected, self.filtered().len());
    }

    pub fn move_up(&mut self) {
        self.selected = list_nav::prev(self.selected);
    }

    pub fn rebuild(&mut self, entries: Vec<BufferEntry>) {
        self.entries = entries;
        self.clamp();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn window(&self, height: usize) -> (usize, Vec<&BufferEntry>) {
        if height == 0 {
            return (0, Vec::new());
        }
        let offset = list_nav::scroll_offset(self.selected, height);
        let visible = self
            .filtered()
            .into_iter()
            .skip(offset)
            .take(height)
            .collect();
        (offset, visible)
    }

    fn clamp(&mut self) {
        let len = self.filtered().len();
        self.selected = self.selected.min(len.saturating_sub(1));
    }
}
