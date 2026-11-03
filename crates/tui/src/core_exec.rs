mod observer;

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
    with_navigation(app, run).0
}

pub(crate) fn with_navigation<T>(app: &mut App, run: impl FnOnce(&Context<'_>) -> T) -> (T, bool) {
    run_context(app, run, true)
}

#[cfg(feature = "lua")]
pub(crate) fn config_context<T>(app: &mut App, run: impl FnOnce(&Context<'_>) -> T) -> T {
    run_context(app, run, false).0
}

fn run_context<T>(app: &mut App, run: impl FnOnce(&Context<'_>) -> T, drain: bool) -> (T, bool) {
    let old = app.workspace.current_id();
    let bus = app.modules.autocmd_bus();
    let observer = observer::Observer::new(app, bus);
    #[cfg(feature = "lua")]
    let host = app
        .modules
        .get::<v_ide_lua::module::LuaModule>()
        .and_then(v_ide_lua::module::LuaModule::owned_host);
    let (result, messages, navigated) = {
        let cx = Context::with_observer(&mut app.workspace, &observer);
        (run(&cx), cx.take_messages(), cx.navigated())
    };
    reconcile(app, old);
    app.sync_timeout();
    for message in messages {
        app.ui.set_message(message);
    }
    let navigated = navigated || old != app.workspace.current_id();
    if !drain {
        return (result, navigated);
    }
    #[cfg(feature = "lua")]
    if let Some(host) = host {
        use v_ide_editor::module::Request;
        let mut requests: Vec<_> = host
            .take_creates()
            .into_iter()
            .map(|name| Request::Command(format!("new {name}")))
            .collect();
        requests.extend(
            host.take_opens()
                .into_iter()
                .map(|path| Request::Open(app.workspace.cwd().join(path))),
        );
        requests.extend(
            host.take_external_opens()
                .into_iter()
                .map(Request::OpenExternal),
        );
        let opened = crate::requests::apply_navigation(app, requests);
        return (result, navigated || opened);
    }
    (result, navigated)
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
