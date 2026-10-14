use std::path::PathBuf;

use v_ide_editor::module::{Cx, Event, Manifest, Module, Modules, Registration, Trigger};

struct Echo;

impl Module for Echo {
    fn activate(&mut self, cx: &mut Cx) -> Result<(), String> {
        cx.message("activate");
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, name: &str, args: &str) -> Result<(), String> {
        cx.message(format!("{name} {args}"));
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

struct Broken;

impl Module for Broken {
    fn activate(&mut self, _cx: &mut Cx) -> Result<(), String> {
        Err(String::from("boom"))
    }

    fn command(&mut self, cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        cx.message("ran");
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

struct Faulty;

impl Module for Faulty {
    fn activate(&mut self, _cx: &mut Cx) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        cx.message("partial");
        Err(String::from("bad"))
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

struct Listener;

impl Module for Listener {
    fn activate(&mut self, cx: &mut Cx) -> Result<(), String> {
        cx.message("listen");
        Ok(())
    }

    fn command(&mut self, _cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        Ok(())
    }

    fn event(&mut self, cx: &mut Cx, ev: &Event) {
        cx.message(format!("event {}", ev.name()));
    }
}

struct Boot;

impl Module for Boot {
    fn activate(&mut self, cx: &mut Cx) -> Result<(), String> {
        cx.message("up");
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        cx.message("boot");
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

struct Effects;

impl Module for Effects {
    fn activate(&mut self, _cx: &mut Cx) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        cx.open(PathBuf::from("a.md"));
        cx.pick(vec![String::from("x")], "choose");
        cx.list("Title", vec![String::from("r")], "go");
        cx.emit(Event::NoteSave(PathBuf::from("a.md")));
        let info = format!("{:?} {}", cx.path(), cx.text());
        cx.message(info);
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

pub struct Tally {
    pub count: usize,
}

impl Module for Tally {
    fn activate(&mut self, _cx: &mut Cx) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        self.count += 1;
        cx.message(self.count.to_string());
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

struct Where;

impl Module for Where {
    fn activate(&mut self, _cx: &mut Cx) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        let cwd = cx.cwd().display().to_string();
        cx.message(cwd);
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx, _ev: &Event) {}
}

static ECHO: Manifest = Manifest {
    name: "echo",
    commands: &["echo"],
    triggers: &[],
};
static BROKEN: Manifest = Manifest {
    name: "broken",
    commands: &["broken"],
    triggers: &[],
};
static FAULTY: Manifest = Manifest {
    name: "faulty",
    commands: &["faulty"],
    triggers: &[],
};
static LISTENER: Manifest = Manifest {
    name: "listener",
    commands: &[],
    triggers: &[Trigger::Event("note_open")],
};
static BOOT: Manifest = Manifest {
    name: "boot",
    commands: &["boot"],
    triggers: &[Trigger::Startup],
};
static EFFECTS: Manifest = Manifest {
    name: "effects",
    commands: &["effects"],
    triggers: &[],
};
static TALLY: Manifest = Manifest {
    name: "tally",
    commands: &["tally"],
    triggers: &[],
};
static WHERE: Manifest = Manifest {
    name: "where",
    commands: &["where"],
    triggers: &[],
};

fn reg(manifest: &'static Manifest, make: fn() -> Box<dyn Module>) -> Registration {
    Registration { manifest, make }
}

pub fn all() -> Modules {
    Modules::new(vec![
        reg(&ECHO, || Box::new(Echo)),
        reg(&BROKEN, || Box::new(Broken)),
        reg(&FAULTY, || Box::new(Faulty)),
        reg(&LISTENER, || Box::new(Listener)),
        reg(&BOOT, || Box::new(Boot)),
        reg(&EFFECTS, || Box::new(Effects)),
        reg(&TALLY, || Box::new(Tally { count: 0 })),
        reg(&WHERE, || Box::new(Where)),
    ])
}
