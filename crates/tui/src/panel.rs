use crate::list_nav;
use crate::list_panel::ListPanel;
use v_ide_notebook::tree::{Row, Tree};

pub enum Panel {
    Tree { tree: Tree, selected: usize },
    List(ListPanel),
}

pub enum Activation {
    Open(String),
    Run { command: String, item: String },
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

    pub fn title(&self) -> String {
        match self {
            Panel::Tree { .. } => String::from(" tree "),
            Panel::List(list) => format!(" {} ", list.title()),
        }
    }

    pub fn selected(&self) -> usize {
        match self {
            Panel::Tree { selected, .. } => *selected,
            Panel::List(list) => list.selected(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Panel::Tree { tree, .. } => tree.flatten().len(),
            Panel::List(list) => list.rows().len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn rows(&self) -> Vec<String> {
        match self {
            Panel::Tree { tree, .. } => tree.flatten().iter().map(row_label).collect(),
            Panel::List(list) => list.rows().to_vec(),
        }
    }

    pub fn move_down(&mut self) {
        let len = self.len();
        match self {
            Panel::Tree { selected, .. } => *selected = list_nav::next(*selected, len),
            Panel::List(list) => list.move_down(),
        }
    }

    pub fn move_up(&mut self) {
        match self {
            Panel::Tree { selected, .. } => *selected = list_nav::prev(*selected),
            Panel::List(list) => list.move_up(),
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
            Panel::List(list) => match list.current() {
                Some(item) => Activation::Run {
                    command: list.command().to_string(),
                    item: item.to_string(),
                },
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
