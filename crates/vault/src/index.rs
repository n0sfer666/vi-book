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
        let names = names_of(&notes);
        Self { notes, names }
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn names(&self) -> &Names {
        &self.names
    }

    pub(crate) fn upsert(&mut self, note: Note) -> bool {
        match self.notes.binary_search_by(|n| n.path.cmp(&note.path)) {
            Ok(at) if self.notes[at] == note => false,
            Ok(at) => {
                self.notes[at] = note;
                true
            }
            Err(at) => {
                self.names.insert(note.path.clone());
                self.notes.insert(at, note);
                true
            }
        }
    }

    pub(crate) fn apply(&mut self, notes: Vec<Note>, gone: &BTreeSet<String>) -> bool {
        let before = self.notes.len();
        self.notes.retain(|note| !gone.contains(&note.path));
        let mut changed = false;
        let mut fresh = Vec::new();
        for note in notes {
            match self.notes.binary_search_by(|n| n.path.cmp(&note.path)) {
                Ok(at) if self.notes[at] == note => {}
                Ok(at) => {
                    self.notes[at] = note;
                    changed = true;
                }
                Err(_) => fresh.push(note),
            }
        }
        let moved = self.notes.len() != before || !fresh.is_empty();
        if moved {
            self.notes.extend(fresh);
            self.notes.sort_by(|a, b| a.path.cmp(&b.path));
            self.names = names_of(&self.notes);
        }
        changed || moved
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

fn names_of(notes: &[Note]) -> Names {
    Names::from_paths(notes.iter().map(|note| note.path.clone()).collect())
}
