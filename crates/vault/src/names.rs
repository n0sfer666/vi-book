use std::collections::{BTreeSet, HashMap};

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

    pub(crate) fn insert(&mut self, path: String) {
        if let Err(at) = self.paths.binary_search(&path) {
            let mut paths = std::mem::take(&mut self.paths);
            paths.insert(at, path);
            *self = Self::from_paths(paths);
        }
    }

    pub(crate) fn apply<'a>(
        &mut self,
        found: impl Iterator<Item = &'a String>,
        gone: &BTreeSet<String>,
    ) {
        let fresh: Vec<&String> = found
            .filter(|path| self.paths.binary_search(path).is_err())
            .collect();
        if fresh.is_empty() && !self.paths.iter().any(|path| gone.contains(path)) {
            return;
        }
        let mut paths: Vec<String> = std::mem::take(&mut self.paths)
            .into_iter()
            .filter(|path| !gone.contains(path))
            .chain(fresh.into_iter().cloned())
            .collect();
        paths.sort();
        *self = Self::from_paths(paths);
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
