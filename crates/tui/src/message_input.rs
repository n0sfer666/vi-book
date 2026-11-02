use std::time::Instant;
use v_ide_engine::Mode;
use v_ide_keys::{Code, Key, Mods};

use crate::{app::App, pager::PagerInput};

pub(crate) fn handle(app: &mut App, key: Key) {
    if let PagerInput::Replay(key) = apply(app, key) {
        app.typeahead.feed_front(key);
    }
    crate::key_feed::timeout(app, Instant::now());
}

pub(crate) fn apply(app: &mut App, key: Key) -> PagerInput {
    let Some(pager) = app.ui.pager.as_mut() else {
        return PagerInput::Replay(key);
    };
    let was_more = pager.more();
    let mut input = pager.key(key);
    if input == PagerInput::Command && !was_more {
        input = PagerInput::Replay(key);
    }
    if input != PagerInput::Keep {
        app.ui.pager = None;
        app.ui.message.clear();
    }
    if input == PagerInput::Command {
        command(app);
    }
    input
}

fn command(app: &mut App) {
    if let Some(draft) = app.ui.command.as_mut() {
        draft.push(':');
        return;
    }
    let editor = &mut app.workspace.current_mut().editor;
    if editor.search_input().is_some() {
        editor.handle_key(Key::char(':'));
        return;
    }
    let mode = editor.mode();
    editor.handle_key(Key::new(Code::Esc, Mods::NONE));
    if mode != Mode::Normal {
        app.typeahead.flush_mapped();
    }
    match mode {
        Mode::Insert => {}
        Mode::Normal => app.ui.command = Some(String::new()),
        Mode::Visual | Mode::VisualLine | Mode::VisualBlock => {
            app.ui.command = Some(String::from("'<,'>"));
        }
    }
}
