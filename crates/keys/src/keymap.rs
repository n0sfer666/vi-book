use std::collections::HashMap;

use crate::empty_lhs::EmptyLhs;
use crate::key::Key;
use crate::map_mode::MapMode;
use crate::mapping::Mapping;
use crate::trie::Trie;

pub struct Keymap<A> {
    tables: HashMap<MapMode, Trie<Mapping<A>>>,
}

impl<A> Default for Keymap<A> {
    fn default() -> Self {
        Keymap {
            tables: HashMap::new(),
        }
    }
}

impl<A> Keymap<A> {
    pub fn insert(
        &mut self,
        mode: MapMode,
        lhs: &[Key],
        mapping: Mapping<A>,
    ) -> Result<Option<Mapping<A>>, EmptyLhs> {
        if lhs.is_empty() {
            return Err(EmptyLhs);
        }
        Ok(self.tables.entry(mode).or_default().insert(lhs, mapping))
    }

    pub fn remove(&mut self, mode: MapMode, lhs: &[Key]) -> Option<Mapping<A>> {
        if lhs.is_empty() {
            return None;
        }
        self.tables.get_mut(&mode)?.remove(lhs)
    }

    pub fn clear(&mut self, mode: MapMode) {
        self.tables.remove(&mode);
    }

    pub(crate) fn lookup(
        &self,
        mode: MapMode,
        keys: &[Key],
    ) -> (Option<(usize, &Mapping<A>)>, bool) {
        self.tables
            .get(&mode)
            .map_or((None, false), |table| table.lookup(keys))
    }
}
