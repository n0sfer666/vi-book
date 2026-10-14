use std::collections::HashMap;

use v_ide_editor::document::Document;
use v_ide_editor::module::{Cx, Event, Manifest, Module, Modules, Registration, Request, Trigger};
use v_ide_editor::workspace::Workspace;

struct Shell;

impl Module for Shell {
    fn activate(&mut self, cx: &mut Cx) -> Result<(), String> {
        cx.message(format!("cursor {}", cx.cursor()));
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, name: &str, args: &str) -> Result<(), String> {
        cx.message(format!("{name} {args}"));
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}

    fn commands(&self) -> Vec<String> {
        vec![String::from("run-me")]
    }
}

static SHELL: Manifest = Manifest {
    name: "shell",
    commands: &[],
    triggers: &[Trigger::Startup],
};

struct Static;

impl Module for Static {
    fn activate(&mut self, _cx: &mut Cx) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        cx.message("static");
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

static STATIC: Manifest = Manifest {
    name: "static",
    commands: &["run-me"],
    triggers: &[],
};

static DORMANT: Manifest = Manifest {
    name: "dormant",
    commands: &[],
    triggers: &[],
};

fn ws() -> Workspace {
    let mut ws = Workspace::new(Document::new("hello"));
    ws.current_mut()
        .editor
        .handle_key(v_ide_keys::Key::Char('$'));
    ws
}

fn modules(manifest: &'static Manifest) -> Modules {
    Modules::new(vec![Registration {
        manifest,
        make: || Box::new(Shell),
    }])
}

fn msg(text: &str) -> Request {
    Request::Message(String::from(text))
}

#[test]
fn cx_exposes_cursor_index() {
    assert_eq!(modules(&SHELL).startup(&ws()), vec![msg("cursor 4")]);
}

#[test]
fn active_module_handles_its_dynamic_commands() {
    let (mut modules, ws) = (modules(&SHELL), ws());
    modules.startup(&ws);
    assert_eq!(
        modules.command(&ws, "run-me", "now"),
        Some(vec![msg("run-me now")])
    );
    assert_eq!(modules.command(&ws, "other", ""), None);
}

#[test]
fn inactive_module_dynamic_commands_are_unknown() {
    assert_eq!(modules(&DORMANT).command(&ws(), "run-me", ""), None);
}

#[test]
fn new_events_are_named() {
    let changed = Event::VaultChanged {
        notes: Vec::new(),
        tags: Vec::new(),
        backlinks: HashMap::new(),
    };
    assert_eq!(changed.name(), "vault_changed");
    assert_eq!(
        Event::FollowLink(String::from("a.rs")).name(),
        "follow_link"
    );
}

#[test]
fn manifest_command_wins_over_earlier_dynamic_one() {
    let mut modules = Modules::new(vec![
        Registration {
            manifest: &SHELL,
            make: || Box::new(Shell),
        },
        Registration {
            manifest: &STATIC,
            make: || Box::new(Static),
        },
    ]);
    let ws = ws();
    modules.startup(&ws);
    assert_eq!(
        modules.command(&ws, "run-me", ""),
        Some(vec![msg("static")])
    );
}
