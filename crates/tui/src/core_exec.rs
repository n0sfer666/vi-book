use crate::app::App;
use v_ide_editor::execution::{Context, Error};

pub(crate) fn execute(app: &mut App, input: &str) -> Option<bool> {
    if matches!(input, "w" | "w!" | "wq" | "e!")
        || input.starts_with("e ")
        || !Context::supports(input)
    {
        return None;
    }
    let (result, messages) = with_context(app, |cx| (cx.execute(input), cx.take_messages()));
    if !messages.is_empty() {
        app.ui.show_output(messages.join("\n"));
    }
    match result {
        Ok(()) => Some(true),
        Err(Error::Autocmd(_)) => Some(false),
        Err(error) => {
            app.ui.set_message(error.to_string());
            Some(false)
        }
    }
}

pub(crate) fn with_context<T>(app: &mut App, run: impl FnOnce(&Context<'_>) -> T) -> T {
    let old = app.workspace.current_id();
    let bus = app.modules.autocmd_bus();
    let (result, messages) = {
        let cx = Context::with_observer(&mut app.workspace, bus.as_ref());
        (run(&cx), cx.take_messages())
    };
    reconcile(app, old);
    app.sync_timeout();
    for message in messages {
        app.ui.set_message(message);
    }
    result
}

fn reconcile(app: &mut App, old: v_ide_editor::workspace::DocId) {
    if old == app.workspace.current_id() {
        return;
    }
    let window = app.workspace.active_window();
    app.ui.read.left = app.ui.left;
    let state = std::mem::take(&mut app.ui.read);
    if app.workspace.doc(old).is_some() {
        app.read_states.insert((window, old), state);
    }
    app.ui.read = app
        .read_states
        .remove(&(window, app.workspace.current_id()))
        .unwrap_or_else(|| {
            let mut state = crate::read_state::ReadState::new();
            state.view = app.initial_view();
            state
        });
    app.ui.left = app.ui.read.left;
}
