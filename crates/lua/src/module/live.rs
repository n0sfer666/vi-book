use super::LuaModule;
use v_ide_editor::module::{Action, Cx, Event, Request};

pub(super) enum Call {
    Initialize,
    Command(String),
    Event(Event),
}

impl LuaModule {
    pub(super) fn prepare(&self, cx: &Cx<'_>, call: Call) -> Option<Action> {
        let host = self.owned_host()?;
        let vault = cx.vault();
        let cwd = cx.cwd().to_path_buf();
        Some(Action::new(move |context| {
            if let Some(vault) = &vault {
                host.set_vault(vault.clone());
            }
            let result = host.call_with_context(context, |host| match &call {
                Call::Initialize => {
                    for (_, source) in crate::defaults::ALL {
                        host.load(source)?;
                    }
                    Ok(())
                }
                Call::Command(name) => host.run_command(name).map(drop),
                Call::Event(Event::FollowLink(target)) => host.fire_with("follow_link", target),
                Call::Event(event) => host.fire(event.name()),
            });
            let creates = host.take_creates();
            let mut opens = host.take_opens();
            let mut externals = host.take_external_opens();
            if let Err(error) = result {
                if error.is_reported() {
                    return Vec::new();
                }
                return vec![Request::Message(format!("plugin error: {error}"))];
            }
            let mut requests: Vec<_> = creates
                .into_iter()
                .map(|name| Request::Command(format!("new {name}")))
                .collect();
            if matches!(&call, Call::Event(Event::FollowLink(_))) {
                if let Some(path) = externals.pop() {
                    requests.push(Request::OpenExternal(path));
                } else if let Some(path) = opens.pop() {
                    requests.push(Request::Open(cwd.join(path)));
                }
            } else {
                requests.extend(opens.into_iter().map(|path| Request::Open(cwd.join(path))));
                requests.extend(externals.into_iter().map(Request::OpenExternal));
            }
            requests
        }))
    }
}
