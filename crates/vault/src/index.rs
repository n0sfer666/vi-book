use std::collections::BTreeSet;

pub use crate::names::Resolution;

use crate::names::Names;
use crate::note::{self, Note};

pub struct Index {
    notes: Vec<Note>,
    names: Names,
}

impl Index {
    pub fn from_notes(notes: Vec<Note>) -> Self {
        let names = Names::from_paths(notes.iter().map(|note| note.path.clone()).collect());
        Self { notes, names }
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn names(&self) -> &Names {
        &self.names
    }

    pub(crate) fn upsert(&mut self, note: Note) {
        match self.notes.binary_search_by(|n| n.path.cmp(&note.path)) {
            Ok(at) => self.notes[at] = note,
            Err(at) => {
                self.names.insert(note.path.clone());
                self.notes.insert(at, note);
            }
        }
    }

    pub fn resolve(&self, target: &str) -> Resolution {
        self.names.resolve(target)
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
                if !self.names.contains(&link.target) {
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
