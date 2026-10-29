use std::time::Instant;

use crossterm::event::KeyEvent;

use crate::app::App;
use crate::input_target::{InputTarget, target};
use crate::map_action::MapAction;
use crate::{command_line, event_map, router};
use v_ide_keys::{MapMode, Step};

const MAX_STEPS: usize = 100_000;
const RECURSIVE: &str = "E223: Recursive mapping";

pub fn feed(app: &mut App, event: KeyEvent) {
    let Some(stroke) = event_map::to_stroke(event) else {
        return;
    };
    app.ui.message.clear();
    app.typeahead.feed(stroke.key);
    drain(app, Instant::now());
}

pub fn timeout(app: &mut App, now: Instant) {
    drain(app, now);
}

fn drain(app: &mut App, now: Instant) {
    while !settle(app, now) {
        app.typeahead.flush_mapped();
        app.ui.message = String::from(RECURSIVE);
    }
}

fn settle(app: &mut App, now: Instant) -> bool {
    let mut steps = 0;
    while steps < MAX_STEPS {
        let (mode, no_zero) = map_mode(app);
        match app.typeahead.next(&app.keymap, mode, no_zero, now) {
            Step::Key(key) => router::handle(app, key),
            Step::Action(MapAction::Command(command)) => {
                app.workspace.current_mut().editor.discard_count();
                command_line::run(app, &command);
            }
            Step::Recursive => {
                app.ui.message = String::from(RECURSIVE);
                continue;
            }
            Step::Wait | Step::Idle => return true,
        }
        steps += 1;
    }
    false
}

fn map_mode(app: &App) -> (Option<MapMode>, bool) {
    match target(app) {
        InputTarget::Command => (Some(MapMode::Cmdline), false),
        InputTarget::Editor => {
            let editor = &app.workspace.current().editor;
            (editor.map_mode(), editor.counting())
        }
        _ => (None, false),
    }
}
