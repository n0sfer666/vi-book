use std::collections::HashMap;

use v_ide_editor::module::Event;

use crate::index::Index;
use crate::note::stem;

pub fn changed(index: &Index) -> Event {
    let mut by_target: HashMap<String, Vec<String>> = HashMap::new();
    for source in index.notes() {
        for link in &source.links {
            let sources = by_target.entry(link.target.clone()).or_default();
            if sources.last() != Some(&source.path) {
                sources.push(source.path.clone());
            }
        }
    }
    let backlinks = index
        .notes()
        .iter()
        .filter_map(|note| {
            let sources: Vec<String> = by_target
                .get(&stem(&note.path))?
                .iter()
                .filter(|source| **source != note.path)
                .cloned()
                .collect();
            (!sources.is_empty()).then(|| (note.path.clone(), sources))
        })
        .collect();
    Event::VaultChanged {
        notes: index.notes().iter().map(|n| n.path.clone()).collect(),
        tags: index.tags(),
        backlinks,
    }
}
