use std::collections::HashMap;

use crate::key::Key;

pub(crate) struct Trie<V> {
    value: Option<V>,
    children: HashMap<Key, Trie<V>>,
}

impl<V> Default for Trie<V> {
    fn default() -> Self {
        Trie {
            value: None,
            children: HashMap::new(),
        }
    }
}

impl<V> Trie<V> {
    pub(crate) fn insert(&mut self, keys: &[Key], value: V) -> Option<V> {
        let node = keys
            .iter()
            .fold(self, |node, key| node.children.entry(*key).or_default());
        node.value.replace(value)
    }

    pub(crate) fn remove(&mut self, keys: &[Key]) -> Option<V> {
        let Some((first, rest)) = keys.split_first() else {
            return self.value.take();
        };
        let child = self.children.get_mut(first)?;
        let removed = child.remove(rest);
        if child.value.is_none() && child.children.is_empty() {
            self.children.remove(first);
        }
        removed
    }

    pub(crate) fn lookup(&self, keys: &[Key]) -> (Option<(usize, &V)>, bool) {
        let mut node = self;
        let mut exact = None;
        for (depth, key) in keys.iter().enumerate() {
            let Some(child) = node.children.get(key) else {
                return (exact, false);
            };
            node = child;
            if let Some(value) = &node.value {
                exact = Some((depth + 1, value));
            }
        }
        (exact, !node.children.is_empty())
    }
}
