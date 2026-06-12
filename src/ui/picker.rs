use crate::vault::fuzzy;

pub struct Picker {
    items: Vec<String>,
    query: String,
    selected: usize,
}

impl Picker {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            query: String::new(),
            selected: 0,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn filtered(&self) -> Vec<&str> {
        fuzzy::filter(&self.query, &self.items)
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.clamp();
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
        let len = self.filtered().len();
        if len > 0 {
            self.selected = (self.selected + 1).min(len - 1);
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn current(&self) -> Option<String> {
        self.filtered().get(self.selected).map(|s| s.to_string())
    }

    fn clamp(&mut self) {
        let len = self.filtered().len();
        self.selected = self.selected.min(len.saturating_sub(1));
    }
}
