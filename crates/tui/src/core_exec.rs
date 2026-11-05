mod observer;

use crate::{
    app::App,
    requests::{self, Outcome},
};
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
    with_outcome(app, run).0
}

pub(crate) fn with_outcome<T>(app: &mut App, run: impl FnOnce(&Context<'_>) -> T) -> (T, Outcome) {
    let (result, outcome, _) = run_context(app, run, false, |_, _| {});
    (result, outcome)
}

pub(crate) fn with_buffer_context<T>(
    app: &mut App,
    target: v_ide_editor::workspace::DocId,
    run: impl FnOnce(&Context<'_>) -> T,
) -> (Option<T>, Outcome) {
    let previous = app.workspace.current_id();
    let Some(scope) = app.workspace.enter_event_scope(target) else {
        return (None, Outcome::default());
    };
    if previous != target {
        reconcile(app, previous);
    }
    let (result, mut outcome) = with_outcome(app, run);
    let explicit_navigation = outcome.navigated;
    if app
        .workspace
        .finish_event_scope(scope, explicit_navigation)
        .is_some()
    {
        reconcile(app, target);
    }
    outcome.merge(Outcome::navigation(previous != app.workspace.current_id()));
    (Some(result), outcome)
}

#[cfg(feature = "lua")]
pub(crate) fn config_context<T>(
    app: &mut App,
    run: impl FnOnce(&Context<'_>) -> T,
) -> (T, Vec<v_ide_editor::execution::Notification>) {
    let (result, _, events) = run_context(app, run, true, |_, _| {});
    (result, events)
}

pub(crate) fn with_after<T>(
    app: &mut App,
    run: impl FnOnce(&Context<'_>) -> T,
    after: impl FnOnce(&mut App, &T),
) -> T {
    run_context(app, run, false, after).0
}

fn run_context<T>(
    app: &mut App,
    run: impl FnOnce(&Context<'_>) -> T,
    defer_events: bool,
    after: impl FnOnce(&mut App, &T),
) -> (T, Outcome, Vec<v_ide_editor::execution::Notification>) {
    let old = app.workspace.current_id();
    let bus = app.modules.autocmd_bus();
    let observer = observer::Observer::new(app, bus);
    #[cfg(feature = "lua")]
    let host = app
        .modules
        .get::<v_ide_lua::module::LuaModule>()
        .and_then(v_ide_lua::module::LuaModule::owned_host);
    let (result, messages, navigated, events) = {
        let cx = Context::with_observer(&mut app.workspace, &observer);
        (
            run(&cx),
            cx.take_messages(),
            cx.navigated(),
            cx.take_events(),
        )
    };
    reconcile(app, old);
    app.sync_timeout();
    after(app, &result);
    for message in messages {
        app.ui.set_message(message);
    }
    let mut outcome = Outcome::navigation(navigated || old != app.workspace.current_id());
    if defer_events {
        return (result, outcome, events);
    }
    #[cfg(feature = "lua")]
    let pending = pending_host_requests(&host, app.workspace.cwd());
    #[cfg(not(feature = "lua"))]
    let pending = Vec::new();
    outcome.merge(finish_events(app, events));
    outcome.merge(requests::apply_outcome(app, pending));
    outcome.merge(Outcome::navigation(old != app.workspace.current_id()));
    (result, outcome, Vec::new())
}

pub(crate) fn finish_events(
    app: &mut App,
    events: Vec<v_ide_editor::execution::Notification>,
) -> Outcome {
    let mut outcome = Outcome::default();
    for notification in events {
        let previous = app.workspace.current_id();
        let Some(scope) = app.workspace.enter_event_scope(notification.buffer) else {
            continue;
        };
        let target = notification.buffer;
        if previous != target {
            reconcile(app, previous);
        }
        let effects = requests::emit_outcome(app, notification.event);
        let explicit_navigation = effects.navigated;
        outcome.merge(effects);
        if app
            .workspace
            .finish_event_scope(scope, explicit_navigation)
            .is_some()
        {
            reconcile(app, target);
        }
        outcome.merge(Outcome::navigation(previous != app.workspace.current_id()));
    }
    outcome
}

#[cfg(feature = "lua")]
fn pending_host_requests(
    host: &Option<std::rc::Rc<v_ide_lua::host::PluginHost>>,
    cwd: &std::path::Path,
) -> Vec<v_ide_editor::module::Request> {
    use v_ide_editor::module::Request;
    let Some(host) = host else {
        return Vec::new();
    };
    let mut requests = host
        .take_creates()
        .into_iter()
        .map(|name| Request::Command(format!("new {name}")))
        .collect::<Vec<_>>();
    requests.extend(
        host.take_opens()
            .into_iter()
            .map(|path| Request::Open(cwd.join(path))),
    );
    requests.extend(
        host.take_external_opens()
            .into_iter()
            .map(Request::OpenExternal),
    );
    requests
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
