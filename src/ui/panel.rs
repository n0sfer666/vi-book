use crate::notebook::tree::{Row, Tree};

pub enum Panel {
    Tree { tree: Tree, selected: usize },
    Backlinks { items: Vec<String>, selected: usize },
}

pub enum Activation {
    Open(String),
    Toggled,
    None,
}

impl Panel {
    pub fn tree(paths: &[String]) -> Self {
        Panel::Tree {
            tree: Tree::from_paths(paths),
            selected: 0,
        }
    }

    pub fn backlinks(items: Vec<String>) -> Self {
        Panel::Backlinks { items, selected: 0 }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Panel::Tree { .. } => " tree ",
            Panel::Backlinks { .. } => " backlinks ",
        }
    }

    pub fn selected(&self) -> usize {
        match self {
            Panel::Tree { selected, .. } | Panel::Backlinks { selected, .. } => *selected,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Panel::Tree { tree, .. } => tree.flatten().len(),
            Panel::Backlinks { items, .. } => items.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn rows(&self) -> Vec<String> {
        match self {
            Panel::Tree { tree, .. } => tree.flatten().iter().map(row_label).collect(),
            Panel::Backlinks { items, .. } => items.clone(),
        }
    }

    pub fn move_down(&mut self) {
        let len = self.len();
        if len == 0 {
            return;
        }
        match self {
            Panel::Tree { selected, .. } | Panel::Backlinks { selected, .. } => {
                *selected = (*selected + 1).min(len - 1);
            }
        }
    }

    pub fn move_up(&mut self) {
        match self {
            Panel::Tree { selected, .. } | Panel::Backlinks { selected, .. } => {
                *selected = selected.saturating_sub(1);
            }
        }
    }

    pub fn activate(&mut self) -> Activation {
        match self {
            Panel::Tree { tree, selected } => {
                let rows = tree.flatten();
                match rows.get(*selected) {
                    Some(row) if row.is_dir => {
                        tree.toggle(&row.path);
                        Activation::Toggled
                    }
                    Some(row) => Activation::Open(row.path.clone()),
                    None => Activation::None,
                }
            }
            Panel::Backlinks { items, selected } => match items.get(*selected) {
                Some(path) => Activation::Open(path.clone()),
                None => Activation::None,
            },
        }
    }
}

fn row_label(row: &Row) -> String {
    let indent = "  ".repeat(row.depth);
    let marker = if row.is_dir {
        if row.collapsed { "▸ " } else { "▾ " }
    } else {
        ""
    };
    format!("{indent}{marker}{}", row.label)
}
