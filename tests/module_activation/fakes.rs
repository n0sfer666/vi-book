use std::sync::atomic::{AtomicUsize, Ordering};

use v_ide_editor::module::{Cx, Event, Manifest, Module, Modules, Registration, Trigger};

pub static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

struct Flaky;

impl Module for Flaky {
    fn activate(&mut self, _cx: &mut Cx) -> Result<(), String> {
        let attempt = ATTEMPTS.fetch_add(1, Ordering::SeqCst) + 1;
        Err(format!("boom {attempt}"))
    }

    fn command(&mut self, _cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

struct Watcher;

impl Module for Watcher {
    fn activate(&mut self, cx: &mut Cx) -> Result<(), String> {
        cx.message("watch up");
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        cx.message("watch");
        Ok(())
    }

    fn event(&mut self, cx: &mut Cx, ev: &Event) {
        match ev {
            Event::VaultChanged { notes, .. } => cx.message(format!("vault {}", notes.join(","))),
            other => cx.message(format!("seen {}", other.name())),
        }
    }
}

struct Md;

impl Module for Md {
    fn activate(&mut self, cx: &mut Cx) -> Result<(), String> {
        cx.message("md up");
        Ok(())
    }

    fn command(&mut self, _cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

static FLAKY: Manifest = Manifest {
    name: "flaky",
    commands: &["flaky"],
    triggers: &[Trigger::Event("note_save")],
};
static WATCHER: Manifest = Manifest {
    name: "watcher",
    commands: &["watch"],
    triggers: &[
        Trigger::Observe("vault_changed"),
        Trigger::Observe("note_open"),
    ],
};
static MD: Manifest = Manifest {
    name: "md",
    commands: &[],
    triggers: &[Trigger::Filetype("markdown")],
};

pub fn modules() -> Modules {
    Modules::new(vec![
        Registration {
            manifest: &FLAKY,
            make: || Box::new(Flaky),
        },
        Registration {
            manifest: &WATCHER,
            make: || Box::new(Watcher),
        },
        Registration {
            manifest: &MD,
            make: || Box::new(Md),
        },
    ])
}
