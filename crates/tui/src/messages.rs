use std::collections::VecDeque;

use crate::app::App;

#[derive(Default)]
pub struct Messages {
    history: VecDeque<String>,
    pending: Vec<String>,
    depth: usize,
}

impl Messages {
    pub fn history(&self) -> impl Iterator<Item = &str> {
        self.history.iter().map(String::as_str)
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }

    pub(crate) fn output(&mut self, text: &str, remember: bool) -> bool {
        if !text.is_empty() {
            if remember {
                if self.history.len() == 1000 {
                    self.history.pop_front();
                }
                self.history.push_back(text.to_string());
            }
            if self.depth > 0 {
                self.pending.push(text.to_string());
            }
        }
        self.depth == 0
    }

    pub(crate) fn begin(&mut self) {
        self.depth += 1;
    }

    pub(crate) fn end(&mut self) -> Option<String> {
        self.depth -= 1;
        if self.depth == 0 && !self.pending.is_empty() {
            Some(std::mem::take(&mut self.pending).join("\n"))
        } else {
            None
        }
    }
}

pub(crate) fn batch<T>(app: &mut App, run: impl FnOnce(&mut App) -> T) -> T {
    app.ui.messages.begin();
    let result = run(app);
    if let Some(output) = app.ui.messages.end()
        && app.workspace.current().editor.search_input().is_none()
    {
        app.ui.present(output);
    }
    result
}
