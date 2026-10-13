#[derive(Default)]
pub struct History {
    entries: Vec<String>,
    cursor: usize,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> Option<&str> {
        self.entries.get(self.cursor).map(String::as_str)
    }

    pub fn visit(&mut self, path: &str) {
        if self.current() == Some(path) {
            return;
        }
        if !self.entries.is_empty() {
            self.entries.truncate(self.cursor + 1);
        }
        self.entries.push(path.to_string());
        self.cursor = self.entries.len() - 1;
    }

    pub fn back(&mut self) -> Option<String> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        self.current().map(str::to_string)
    }

    pub fn forward(&mut self) -> Option<String> {
        if self.cursor + 1 >= self.entries.len() {
            return None;
        }
        self.cursor += 1;
        self.current().map(str::to_string)
    }
}
