use std::collections::HashMap;

use crate::note::stem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    None,
    One(String),
    Many(Vec<String>),
}

pub struct Names {
    paths: Vec<String>,
    by_name: HashMap<String, Vec<usize>>,
}

impl Names {
    pub fn from_paths(paths: Vec<String>) -> Self {
        let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, path) in paths.iter().enumerate() {
            by_name.entry(stem(path)).or_default().push(i);
        }
        Self { paths, by_name }
    }

    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    pub fn contains(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    pub fn resolve(&self, target: &str) -> Resolution {
        let mut paths: Vec<String> = self
            .by_name
            .get(target)
            .into_iter()
            .flatten()
            .map(|&i| self.paths[i].clone())
            .collect();
        paths.sort();
        match paths.len() {
            0 => Resolution::None,
            1 => Resolution::One(paths.remove(0)),
            _ => Resolution::Many(paths),
        }
    }
}
