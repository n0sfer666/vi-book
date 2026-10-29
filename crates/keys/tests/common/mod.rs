#![allow(dead_code)]

use std::time::{Duration, Instant};

use v_ide_keys::{
    Code, Key, Keymap, MapMode, Mapping, Mods, Rhs, Step, TIMEOUT, Typeahead, format_keys,
    parse_keys,
};

pub type Map = Keymap<&'static str>;

pub fn keymap(entries: &[(MapMode, &str, &'static str, bool)]) -> Map {
    let mut map = Keymap::default();
    for &(mode, lhs, rhs, remap) in entries {
        let inserted = map.insert(mode, &parse_keys(lhs), mapping(rhs, remap));
        assert!(inserted.is_ok(), "{lhs}");
    }
    map
}

pub fn mapping(rhs: &'static str, remap: bool) -> Mapping<&'static str> {
    let rhs = match rhs.strip_prefix('{').and_then(|r| r.strip_suffix('}')) {
        Some(action) => Rhs::Action(action),
        None => Rhs::Keys(parse_keys(rhs)),
    };
    Mapping { rhs, remap }
}

pub struct Rig {
    pub map: Map,
    pub ta: Typeahead,
    pub mode: Option<MapMode>,
    pub no_zero: bool,
    pub t0: Instant,
}

impl Rig {
    pub fn new(map: Map) -> Rig {
        Rig {
            map,
            ta: Typeahead::new(TIMEOUT),
            mode: Some(MapMode::Normal),
            no_zero: false,
            t0: Instant::now(),
        }
    }

    pub fn in_mode(mut self, mode: Option<MapMode>) -> Rig {
        self.mode = mode;
        self
    }

    pub fn counting(mut self) -> Rig {
        self.no_zero = true;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Rig {
        self.ta = Typeahead::new(timeout);
        self
    }

    pub fn at(&self, ms: u64) -> Instant {
        self.t0 + Duration::from_millis(ms)
    }

    pub fn feed(&mut self, keys: &str) {
        for key in parse_keys(keys) {
            self.ta.feed(key);
        }
    }

    pub fn type_at(&mut self, keys: &str, ms: u64) -> String {
        let now = self.at(ms);
        let mut out = String::new();
        for key in parse_keys(keys) {
            self.ta.feed(key);
            out.push_str(&self.drain(now));
        }
        out
    }

    pub fn type_keys(&mut self, keys: &str) -> String {
        self.type_at(keys, 0)
    }

    pub fn tick(&mut self, ms: u64) -> String {
        let now = self.at(ms);
        self.drain(now)
    }

    pub fn pending(&self) -> String {
        format_keys(self.ta.pending())
    }

    fn drain(&mut self, now: Instant) -> String {
        let mut out = String::new();
        loop {
            match self.ta.next(&self.map, self.mode, self.no_zero, now) {
                Step::Key(key) => {
                    out.push_str(&format_keys(&[key]));
                    self.mode = next_mode(self.mode, key);
                }
                Step::Action(name) => out.push_str(&format!("{{{name}}}")),
                Step::Recursive => out.push_str("!E223"),
                Step::Wait | Step::Idle => return out,
            }
        }
    }
}

fn next_mode(mode: Option<MapMode>, key: Key) -> Option<MapMode> {
    let leave = matches!(
        (key.mods(), key.code()),
        (Mods::NONE, Code::Esc | Code::Enter)
    );
    match (mode, key.plain_char()) {
        (Some(MapMode::Normal), Some('i')) => Some(MapMode::Insert),
        (Some(MapMode::Normal), Some('v')) => Some(MapMode::Visual),
        (Some(MapMode::Normal), Some('d')) => Some(MapMode::OpPending),
        (Some(MapMode::Normal), Some(':')) => Some(MapMode::Cmdline),
        (Some(MapMode::Normal), Some('f')) => None,
        (Some(MapMode::OpPending) | None, _) => Some(MapMode::Normal),
        (Some(MapMode::Insert | MapMode::Visual | MapMode::Cmdline), _) if leave => {
            Some(MapMode::Normal)
        }
        (other, _) => other,
    }
}
