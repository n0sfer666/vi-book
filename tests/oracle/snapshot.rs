use std::time::Instant;

use v_ide_engine::{Editor, Mode};
use v_ide_keys::{Keymap, Mapping, Rhs, Step, TIMEOUT, Typeahead, parse_keys};
use v_ide_text::Position;

use crate::fixture::{Case, Map, Snapshot};

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Normal => "normal",
        Mode::Insert => "insert",
        Mode::Visual => "visual",
        Mode::VisualLine => "visual_line",
        Mode::VisualBlock => "visual_block",
    }
}

fn keymap(maps: &[Map]) -> Keymap<()> {
    let mut keymap = Keymap::default();
    for map in maps {
        let mapping = Mapping {
            rhs: Rhs::Keys(parse_keys(&map.rhs)),
            remap: !map.noremap,
        };
        let inserted = keymap.insert(map.mode, &parse_keys(&map.lhs), mapping);
        assert!(inserted.is_ok(), "empty lhs {:?}", map.lhs);
    }
    keymap
}

const MAX_STEPS: usize = 100_000;

fn drain(editor: &mut Editor, typeahead: &mut Typeahead, map: &Keymap<()>, flush: bool) -> bool {
    let mut now = Instant::now();
    for _ in 0..MAX_STEPS {
        match typeahead.next(map, editor.map_mode(), editor.counting(), now) {
            Step::Key(key) => editor.handle_key(key),
            Step::Wait if flush => now += TIMEOUT,
            Step::Wait | Step::Idle => return true,
            Step::Action(()) | Step::Recursive => {}
        }
    }
    false
}

pub fn run(case: &Case) -> Snapshot {
    let mut editor = Editor::new(&case.before);
    editor.set_cursor(Position::new(case.cursor[0], case.cursor[1]));
    let map = keymap(&case.maps);
    let mut typeahead = Typeahead::new(TIMEOUT);
    for key in parse_keys(&case.keys) {
        typeahead.feed(key);
        let settled = drain(&mut editor, &mut typeahead, &map, false);
        assert!(settled, "{}: typeahead did not settle", case.name);
    }
    let settled = drain(&mut editor, &mut typeahead, &map, true);
    assert!(settled, "{}: typeahead did not settle", case.name);
    let cursor = editor.cursor();
    Snapshot {
        text: editor.text(),
        cursor: [cursor.line, cursor.col],
        mode: mode_name(editor.mode()).to_string(),
        unnamed: editor.yanked().unwrap_or_default(),
    }
}
