use std::time::{Duration, Instant};

use crate::key::Key;
use crate::keymap::Keymap;
use crate::map_mode::MapMode;
use crate::mapping::Mapping;
use crate::step::Step;
use crate::typeahead_queue::Queue;

pub const TIMEOUT: Duration = Duration::from_millis(1000);

const MAX_DEPTH: usize = 1000;

pub struct Typeahead<A = ()> {
    queue: Queue<A>,
    timeout: Duration,
    deadline: Option<Instant>,
    depth: usize,
}

impl<A: Clone> Typeahead<A> {
    pub fn new(timeout: Duration) -> Self {
        Typeahead {
            queue: Queue::default(),
            timeout,
            deadline: None,
            depth: 0,
        }
    }

    pub fn feed(&mut self, key: Key) {
        self.queue.feed(key);
        self.deadline = None;
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub fn feed_front(&mut self, key: Key) {
        self.queue.feed_front(key);
        self.deadline = None;
    }

    pub fn replay(&mut self, key: Key) {
        self.queue.replay(key);
        self.deadline = None;
    }

    pub fn flush_mapped(&mut self) {
        self.depth = 0;
        if self.queue.flush_mapped() {
            self.deadline = None;
        }
    }

    pub fn pending(&self) -> &[Key] {
        match self.deadline {
            Some(_) => &self.queue.keys,
            None => &[],
        }
    }

    pub fn next(
        &mut self,
        map: &Keymap<A>,
        mode: Option<MapMode>,
        no_zero: bool,
        now: Instant,
    ) -> Step<A> {
        loop {
            if let Some(action) = self.queue.action() {
                self.deadline = None;
                self.depth = 0;
                return Step::Action(action);
            }
            if self.queue.keys.is_empty() {
                self.deadline = None;
                self.depth = 0;
                return Step::Idle;
            }
            let zero = no_zero && self.queue.keys[0] == Key::char('0');
            let window = match zero {
                true => 0,
                false => self.queue.window(),
            };
            let (exact, partial) = match mode {
                Some(mode) if window > 0 => map.lookup(mode, &self.queue.keys[..window]),
                _ => (None, false),
            };
            let timed_out = self.deadline.is_some_and(|deadline| now >= deadline);
            let nowait = exact.is_some_and(|(_, mapping)| mapping.nowait);
            if partial && self.queue.can_wait(window) && !timed_out && !nowait {
                self.deadline.get_or_insert(now + self.timeout);
                return Step::Wait;
            }
            self.deadline = None;
            let Some((len, mapping)) = exact else {
                self.depth = 0;
                return Step::Key(self.queue.pop_key());
            };
            if let Some(step) = self.expand(len, mapping) {
                return step;
            }
        }
    }

    fn expand(&mut self, len: usize, mapping: &Mapping<A>) -> Option<Step<A>> {
        let lhs = self.queue.consume(len);
        self.depth += 1;
        if self.depth >= MAX_DEPTH {
            self.flush_mapped();
            return Some(Step::Recursive);
        }
        self.queue.prepend(&mapping.rhs, mapping.remap, &lhs);
        None
    }
}
