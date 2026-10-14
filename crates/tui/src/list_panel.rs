use crate::list_nav;

pub struct ListPanel {
    title: String,
    rows: Vec<String>,
    command: String,
    selected: usize,
}

impl ListPanel {
    pub fn new(title: &str, rows: Vec<String>, command: &str) -> Self {
        Self {
            title: title.to_string(),
            rows,
            command: command.to_string(),
            selected: 0,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn rows(&self) -> &[String] {
        &self.rows
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn current(&self) -> Option<&str> {
        self.rows.get(self.selected).map(String::as_str)
    }

    pub fn move_down(&mut self) {
        self.selected = list_nav::next(self.selected, self.rows.len());
    }

    pub fn move_up(&mut self) {
        self.selected = list_nav::prev(self.selected);
    }
}
