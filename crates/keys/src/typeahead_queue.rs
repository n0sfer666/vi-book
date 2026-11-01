use std::collections::VecDeque;

use crate::origin::Origin;
use crate::{Key, Rhs};

pub(crate) struct Queue<A> {
    pub(crate) keys: Vec<Key>,
    origins: Vec<Origin>,
    actions: VecDeque<(usize, A)>,
}

impl<A> Default for Queue<A> {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            origins: Vec::new(),
            actions: VecDeque::new(),
        }
    }
}

impl<A> Queue<A> {
    pub(crate) fn feed(&mut self, key: Key) {
        self.keys.push(key);
        self.origins.push(Origin::Typed);
    }

    pub(crate) fn action(&mut self) -> Option<A> {
        if self.actions.front().is_some_and(|(at, _)| *at == 0) {
            self.actions.pop_front().map(|(_, action)| action)
        } else {
            None
        }
    }

    pub(crate) fn window(&self) -> usize {
        let remappable = self.origins.iter().take_while(|o| o.remappable()).count();
        self.actions
            .front()
            .map_or(remappable, |(at, _)| remappable.min(*at))
    }

    pub(crate) fn can_wait(&self, window: usize) -> bool {
        window == self.keys.len() && self.actions.is_empty()
    }

    pub(crate) fn consume(&mut self, len: usize) -> Vec<Key> {
        let keys = self.keys.drain(..len).collect();
        self.origins.drain(..len);
        for (at, _) in &mut self.actions {
            *at -= len;
        }
        keys
    }

    pub(crate) fn pop_key(&mut self) -> Key {
        let key = self.keys.remove(0);
        self.origins.remove(0);
        for (at, _) in &mut self.actions {
            *at -= 1;
        }
        key
    }

    pub(crate) fn flush_mapped(&mut self) -> bool {
        let removed_action = !self.actions.is_empty();
        self.actions.clear();
        let mapped = self
            .origins
            .iter()
            .take_while(|&&o| o != Origin::Typed)
            .count();
        self.consume(mapped);
        mapped > 0 || removed_action
    }
}

impl<A: Clone> Queue<A> {
    pub(crate) fn prepend(&mut self, rhs: &Rhs<A>, remap: bool, lhs: &[Key]) {
        let mut front = Queue::default();
        front.append(rhs);
        let self_prefix = front.keys.starts_with(lhs)
            && front.actions.front().is_none_or(|(at, _)| *at >= lhs.len());
        let origins = (0..front.keys.len()).map(|i| match remap {
            false => Origin::Noremap,
            true if i == 0 && self_prefix => Origin::Noremap,
            true => Origin::Remap,
        });
        for (at, _) in &mut self.actions {
            *at += front.keys.len();
        }
        front.actions.append(&mut self.actions);
        self.actions = front.actions;
        self.origins.splice(..0, origins);
        self.keys.splice(..0, front.keys);
    }

    fn append(&mut self, rhs: &Rhs<A>) {
        match rhs {
            Rhs::Keys(keys) => self.keys.extend_from_slice(keys),
            Rhs::Action(action) => self.actions.push_back((self.keys.len(), action.clone())),
            Rhs::Sequence(parts) => {
                for part in parts {
                    self.append(part);
                }
            }
        }
    }
}
