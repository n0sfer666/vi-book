use std::collections::BTreeSet;
use std::collections::HashMap;

use crate::vault::note::{self, Note};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    None,
    One(String),
    Many(Vec<String>),
}

pub struct Index {
    notes: Vec<Note>,
    by_name: HashMap<String, Vec<usize>>,
}

impl Index {
    pub fn from_notes(notes: Vec<Note>) -> Self {
        let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, note) in notes.iter().enumerate() {
            by_name.entry(note.name.clone()).or_default().push(i);
        }
        Self { notes, by_name }
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn resolve(&self, target: &str) -> Resolution {
        let mut paths: Vec<String> = self
            .by_name
            .get(target)
            .into_iter()
            .flatten()
            .map(|&i| self.notes[i].path.clone())
            .collect();
        paths.sort();
        match paths.len() {
            0 => Resolution::None,
            1 => Resolution::One(paths.remove(0)),
            _ => Resolution::Many(paths),
        }
    }

    pub fn backlinks(&self, path: &str) -> Vec<&Note> {
        let target = note::stem(path);
        self.notes
            .iter()
            .filter(|note| note.path != path && note.links.iter().any(|l| l.target == target))
            .collect()
    }

    pub fn broken_links(&self) -> Vec<(String, String)> {
        let mut broken = Vec::new();
        for note in &self.notes {
            for link in &note.links {
                if matches!(self.resolve(&link.target), Resolution::None) {
                    broken.push((note.path.clone(), link.target.clone()));
                }
            }
        }
        broken
    }

    pub fn tags(&self) -> Vec<String> {
        let set: BTreeSet<String> = self
            .notes
            .iter()
            .flat_map(|note| note.tags.iter().cloned())
            .collect();
        set.into_iter().collect()
    }
}
