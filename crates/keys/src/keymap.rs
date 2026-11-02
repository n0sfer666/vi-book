use std::collections::HashMap;

use crate::empty_lhs::EmptyLhs;
use crate::key::Key;
use crate::map_mode::MapMode;
use crate::mapping::Mapping;
use crate::trie::Trie;
use crate::{MapGroup, format_keys};

struct Entry<A> {
    group: u64,
    mapping: Mapping<A>,
}

pub struct Keymap<A> {
    tables: HashMap<MapMode, Trie<Entry<A>>>,
    next_group: u64,
}

impl<A> Default for Keymap<A> {
    fn default() -> Self {
        Keymap {
            tables: HashMap::new(),
            next_group: 0,
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
        let group = self.new_group();
        Ok(self
            .tables
            .entry(mode)
            .or_default()
            .insert(lhs, Entry { group, mapping })
            .map(|entry| entry.mapping))
    }

    pub fn insert_modes(
        &mut self,
        modes: &[MapMode],
        lhs: &[Key],
        mapping: Mapping<A>,
    ) -> Result<(), EmptyLhs>
    where
        A: Clone,
    {
        if lhs.is_empty() {
            return Err(EmptyLhs);
        }
        let group = self.new_group();
        for &mode in modes {
            self.tables.entry(mode).or_default().insert(
                lhs,
                Entry {
                    group,
                    mapping: mapping.clone(),
                },
            );
        }
        Ok(())
    }

    pub fn contains(&self, mode: MapMode, lhs: &[Key]) -> bool {
        self.lookup(mode, lhs)
            .0
            .is_some_and(|(len, _)| len == lhs.len())
    }

    pub fn remove(&mut self, mode: MapMode, lhs: &[Key]) -> Option<Mapping<A>> {
        if lhs.is_empty() {
            return None;
        }
        self.tables
            .get_mut(&mode)?
            .remove(lhs)
            .map(|entry| entry.mapping)
    }

    pub fn clear(&mut self, mode: MapMode) {
        self.tables.remove(&mode);
    }

    pub fn list(&self, modes: &[MapMode], prefix: &[Key]) -> Vec<MapGroup<'_, A>> {
        let mut groups: HashMap<u64, MapGroup<'_, A>> = HashMap::new();
        for mode in [
            MapMode::Normal,
            MapMode::Visual,
            MapMode::OpPending,
            MapMode::Insert,
            MapMode::Cmdline,
        ] {
            let Some(table) = self.tables.get(&mode) else {
                continue;
            };
            for (lhs, entry) in table.entries(prefix) {
                let group = groups.entry(entry.group).or_insert_with(|| MapGroup {
                    modes: Vec::new(),
                    lhs,
                    mapping: &entry.mapping,
                });
                group.modes.push(mode);
            }
        }
        let mut groups: Vec<_> = groups
            .into_iter()
            .filter(|(_, group)| group.modes.iter().any(|m| modes.contains(m)))
            .collect();
        groups.sort_by_cached_key(|(id, group)| (format_keys(&group.lhs), std::cmp::Reverse(*id)));
        groups.into_iter().map(|(_, group)| group).collect()
    }

    fn new_group(&mut self) -> u64 {
        let group = self.next_group;
        self.next_group += 1;
        group
    }

    pub(crate) fn lookup(
        &self,
        mode: MapMode,
        keys: &[Key],
    ) -> (Option<(usize, &Mapping<A>)>, bool) {
        self.tables.get(&mode).map_or((None, false), |table| {
            let (exact, partial) = table.lookup(keys);
            (exact.map(|(len, entry)| (len, &entry.mapping)), partial)
        })
    }
}
