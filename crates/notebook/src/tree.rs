use std::collections::{BTreeMap, HashSet};

#[derive(Default)]
struct Node {
    children: BTreeMap<String, Node>,
    is_file: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub path: String,
    pub label: String,
    pub depth: usize,
    pub is_dir: bool,
    pub collapsed: bool,
}

pub struct Tree {
    root: Node,
    collapsed: HashSet<String>,
}

impl Tree {
    pub fn from_paths(paths: &[String]) -> Self {
        let mut root = Node::default();
        for path in paths {
            insert(&mut root, path);
        }
        Self {
            root,
            collapsed: HashSet::new(),
        }
    }

    pub fn toggle(&mut self, dir_path: &str) {
        if !self.collapsed.remove(dir_path) {
            self.collapsed.insert(dir_path.to_string());
        }
    }

    pub fn is_collapsed(&self, dir_path: &str) -> bool {
        self.collapsed.contains(dir_path)
    }

    pub fn flatten(&self) -> Vec<Row> {
        let mut rows = Vec::new();
        self.walk(&self.root, "", 0, &mut rows);
        rows
    }

    fn walk(&self, node: &Node, prefix: &str, depth: usize, rows: &mut Vec<Row>) {
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for (name, child) in &node.children {
            if child.is_file {
                files.push((name, child));
            } else {
                dirs.push((name, child));
            }
        }
        for (name, child) in dirs.into_iter().chain(files) {
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            let is_dir = !child.is_file;
            let collapsed = is_dir && self.collapsed.contains(&path);
            rows.push(Row {
                path: path.clone(),
                label: name.clone(),
                depth,
                is_dir,
                collapsed,
            });
            if is_dir && !collapsed {
                self.walk(child, &path, depth + 1, rows);
            }
        }
    }
}

fn insert(root: &mut Node, path: &str) {
    let mut node = root;
    let mut parts = path.split('/').peekable();
    while let Some(part) = parts.next() {
        if part.is_empty() {
            continue;
        }
        let last = parts.peek().is_none();
        let child = node.children.entry(part.to_string()).or_default();
        if last {
            child.is_file = true;
        }
        node = child;
    }
}
