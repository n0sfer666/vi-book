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
    if app.ui.pager.is_some() {
        crate::message_input::handle(app, stroke.key);
        return;
    }
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
        app.ui.set_message(String::from(RECURSIVE));
    }
}

fn settle(app: &mut App, now: Instant) -> bool {
    let mut steps = 0;
    while steps < MAX_STEPS {
        if app.ui.pager.as_ref().is_some_and(|pager| pager.more()) {
            return true;
        }
        let (mode, no_zero) = map_mode(app);
        let builtin = (target(app) == InputTarget::Editor
            && app.workspace.current().editor.idle_normal())
        .then_some((app.leader.as_slice(), MapAction::Menu));
        match app
            .typeahead
            .next_with_builtin(&app.keymap, mode, no_zero, now, builtin)
        {
            Step::Key(key) if app.ui.pager.is_some() => {
                if let crate::pager::PagerInput::Replay(key) = crate::message_input::apply(app, key)
                {
                    app.typeahead.replay(key);
                }
            }
            Step::Key(key) => router::handle(app, key),
            Step::Action(MapAction::Menu) => crate::actions::open_menu(app),
            Step::Action(MapAction::Command(command)) => {
                if app.ui.pager.take().is_some() {
                    app.ui.message.clear();
                }
                app.workspace.current_mut().editor.discard_count();
                command_line::run(app, &command);
            }
            Step::Recursive => {
                app.ui.set_message(String::from(RECURSIVE));
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
