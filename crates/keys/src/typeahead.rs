use std::time::{Duration, Instant};

use crate::key::Key;
use crate::keymap::Keymap;
use crate::map_mode::MapMode;
use crate::mapping::{Mapping, Rhs};
use crate::origin::Origin;
use crate::step::Step;

pub const TIMEOUT: Duration = Duration::from_millis(1000);

const MAX_DEPTH: usize = 1000;

pub struct Typeahead {
    keys: Vec<Key>,
    origins: Vec<Origin>,
    timeout: Duration,
    deadline: Option<Instant>,
    depth: usize,
}

impl Typeahead {
    pub fn new(timeout: Duration) -> Typeahead {
        Typeahead {
            keys: Vec::new(),
            origins: Vec::new(),
            timeout,
            deadline: None,
            depth: 0,
        }
    }

    pub fn feed(&mut self, key: Key) {
        self.keys.push(key);
        self.origins.push(Origin::Typed);
        self.deadline = None;
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub fn pending(&self) -> &[Key] {
        match self.deadline {
            Some(_) => &self.keys,
            None => &[],
        }
    }

    pub fn next<A: Clone>(
        &mut self,
        map: &Keymap<A>,
        mode: Option<MapMode>,
        now: Instant,
    ) -> Step<A> {
        loop {
            if self.keys.is_empty() {
                self.deadline = None;
                self.depth = 0;
                return Step::Idle;
            }
            let window = self.origins.iter().take_while(|o| o.remappable()).count();
            let (exact, partial) = match mode {
                Some(mode) if window > 0 => map.lookup(mode, &self.keys[..window]),
                _ => (None, false),
            };
            let timed_out = self.deadline.is_some_and(|deadline| now >= deadline);
            if partial && window == self.keys.len() && !timed_out {
                self.deadline.get_or_insert(now + self.timeout);
                return Step::Wait;
            }
            self.deadline = None;
            let Some((len, mapping)) = exact else {
                self.depth = 0;
                self.origins.remove(0);
                return Step::Key(self.keys.remove(0));
            };
            if let Some(step) = self.expand(len, mapping) {
                return step;
            }
        }
    }

    fn expand<A: Clone>(&mut self, len: usize, mapping: &Mapping<A>) -> Option<Step<A>> {
        let lhs: Vec<Key> = self.keys.drain(..len).collect();
        self.origins.drain(..len);
        self.depth += 1;
        if self.depth >= MAX_DEPTH {
            self.depth = 0;
            let mapped = self
                .origins
                .iter()
                .take_while(|&&o| o != Origin::Typed)
                .count();
            self.keys.drain(..mapped);
            self.origins.drain(..mapped);
            return Some(Step::Recursive);
        }
        let keys = match &mapping.rhs {
            Rhs::Action(action) => {
                self.depth = 0;
                return Some(Step::Action(action.clone()));
            }
            Rhs::Keys(keys) => keys,
        };
        let origins = keys.iter().enumerate().map(|(i, _)| match mapping.remap {
            false => Origin::Noremap,
            true if i == 0 && keys.starts_with(&lhs) => Origin::Noremap,
            true => Origin::Remap,
        });
        self.origins.splice(..0, origins.collect::<Vec<_>>());
        self.keys.splice(..0, keys.iter().copied());
        None
    }
}
