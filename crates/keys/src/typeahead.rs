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

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.deadline = self
            .deadline
            .and_then(|deadline| deadline.checked_sub(self.timeout))
            .and_then(|start| start.checked_add(timeout));
        self.timeout = timeout;
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
        self.next_with_builtin(map, mode, no_zero, now, None)
    }

    pub fn next_with_builtin(
        &mut self,
        map: &Keymap<A>,
        mode: Option<MapMode>,
        no_zero: bool,
        now: Instant,
        builtin: Option<(&[Key], A)>,
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
            if partial
                && self.queue.can_wait(window)
                && !timed_out
                && !nowait
                && !self.timeout.is_zero()
            {
                self.deadline.get_or_insert(now + self.timeout);
                return Step::Wait;
            }
            let Some((len, mapping)) = exact else {
                if let Some((keys, action)) = &builtin
                    && !keys.is_empty()
                {
                    let window = self.queue.plain_window();
                    let pending = &self.queue.keys[..window];
                    if pending.starts_with(keys) {
                        self.queue.consume(keys.len());
                        self.deadline = None;
                        self.depth = 0;
                        return Step::Action(action.clone());
                    }
                    if keys.starts_with(pending)
                        && self.queue.can_wait(window)
                        && !timed_out
                        && !self.timeout.is_zero()
                    {
                        self.deadline.get_or_insert(now + self.timeout);
                        return Step::Wait;
                    }
                }
                self.deadline = None;
                self.depth = 0;
                return Step::Key(self.queue.pop_key());
            };
            self.deadline = None;
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
