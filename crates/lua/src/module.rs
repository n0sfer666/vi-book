use std::fs;
use std::path::PathBuf;

use v_ide_editor::module::{Cx, Event, Manifest, Module, Registration, Trigger};

use crate::defaults;
use crate::error::Error;
use crate::host::PluginHost;

static MANIFEST: Manifest = Manifest {
    name: "lua",
    commands: &[],
    triggers: &[
        Trigger::Event("follow_link"),
        Trigger::Observe("note_open"),
        Trigger::Observe("note_save"),
    ],
};

pub struct LuaModule {
    host: Option<PluginHost>,
}

impl LuaModule {
    pub fn registration() -> Registration {
        Registration {
            manifest: &MANIFEST,
            make: || Box::new(Self { host: None }),
        }
    }

    pub fn host(&self) -> Option<&PluginHost> {
        self.host.as_ref()
    }

    pub fn load_config(&self, resolved: Option<(PathBuf, Option<&'static str>)>) -> Vec<String> {
        let mut messages = Vec::new();
        let (Some(host), Some((path, warning))) = (&self.host, resolved) else {
            return messages;
        };
        messages.extend(warning.map(String::from));
        if let Ok(script) = fs::read_to_string(&path)
            && let Err(err) = host.load(&script)
        {
            messages.push(format!("config error: {err}"));
        }
        drop(take(host));
        messages
    }

    fn call(
        &self,
        cx: &mut Cx<'_>,
        run: impl FnOnce(&PluginHost) -> Result<(), Error>,
    ) -> Result<(Vec<String>, Vec<String>), String> {
        let Some(host) = &self.host else {
            return Ok((Vec::new(), Vec::new()));
        };
        if let Some(vault) = cx.vault() {
            host.set_vault(vault);
        }
        let before = cx.text();
        host.set_buffer(&before, cx.cursor());
        let result = run(host);
        let (creates, opens, externals) = take(host);
        result.map_err(|err| format!("plugin error: {err}"))?;
        let after = host.buffer_text();
        if after != before {
            cx.set_text(after);
        }
        for name in creates {
            cx.command(format!("new {name}"));
        }
        Ok((opens, externals))
    }

    fn emit_all(
        &self,
        cx: &mut Cx<'_>,
        run: impl FnOnce(&PluginHost) -> Result<(), Error>,
    ) -> Result<(), String> {
        let (opens, externals) = self.call(cx, run)?;
        for rel in opens {
            let abs = cx.cwd().join(rel);
            cx.open(abs);
        }
        for rel in externals {
            cx.open_external(rel);
        }
        Ok(())
    }

    fn emit_follow(&self, cx: &mut Cx<'_>, target: &str) -> Result<(), String> {
        let (mut opens, mut externals) =
            self.call(cx, |host| host.fire_with("follow_link", target))?;
        if let Some(rel) = externals.pop() {
            cx.open_external(rel);
        } else if let Some(rel) = opens.pop() {
            let abs = cx.cwd().join(rel);
            cx.open(abs);
        }
        Ok(())
    }
}

fn take(host: &PluginHost) -> (Vec<String>, Vec<String>, Vec<String>) {
    (
        host.take_creates(),
        host.take_opens(),
        host.take_external_opens(),
    )
}

impl Module for LuaModule {
    fn activate(&mut self, cx: &mut Cx<'_>) -> Result<(), String> {
        let host = PluginHost::new().map_err(|err| format!("plugin init error: {err}"))?;
        for (name, source) in defaults::ALL {
            if let Err(err) = host.load(source) {
                cx.message(format!("default plugin '{name}' error: {err}"));
            }
        }
        drop(take(&host));
        if let Some(vault) = cx.vault() {
            host.set_vault(vault);
        }
        self.host = Some(host);
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx<'_>, name: &str, _args: &str) -> Result<(), String> {
        self.emit_all(cx, |host| host.run_command(name).map(drop))
    }

    fn event(&mut self, cx: &mut Cx<'_>, ev: &Event) {
        let result = match ev {
            Event::NoteOpen(_) | Event::NoteSave(_) => {
                self.emit_all(cx, |host| host.fire(ev.name()))
            }
            Event::FollowLink(target) => self.emit_follow(cx, target),
            Event::VaultChanged { .. } | Event::NoteCreate(_) => Ok(()),
        };
        if let Err(err) = result {
            cx.message(err);
        }
    }

    fn commands(&self) -> Vec<String> {
        self.host
            .as_ref()
            .map(PluginHost::command_names)
            .unwrap_or_default()
    }
}
