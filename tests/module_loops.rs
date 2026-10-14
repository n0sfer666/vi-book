mod common;

use std::fs;
use std::path::PathBuf;

use v_ide_editor::module::{Cx, Event, Manifest, Module, Modules, Registration, Trigger};
use v_ide_tui::app::App;
use v_ide_tui::command_line;

struct Again;

impl Module for Again {
    fn activate(&mut self, _cx: &mut Cx) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        cx.command(String::from("again"));
        Ok(())
    }

    fn event(&mut self, cx: &mut Cx, ev: &Event) {
        if let Event::NoteOpen(path) = ev {
            let other = if path.ends_with("a.md") {
                "b.md"
            } else {
                "a.md"
            };
            let abs = cx.cwd().join(other);
            cx.open(abs);
        }
    }
}

static AGAIN: Manifest = Manifest {
    name: "again",
    commands: &["again"],
    triggers: &[Trigger::Event("note_open")],
};

fn app() -> (App, PathBuf) {
    let root = std::env::temp_dir().join(format!("vi-book-loops-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("b.md"), "# B\n").unwrap();
    let mut app = App::new(common::named_doc("", ""));
    app.workspace.set_cwd(root.clone());
    app.modules = Modules::new(vec![Registration {
        manifest: &AGAIN,
        make: || Box::new(Again),
    }]);
    (app, root)
}

#[test]
fn command_request_loop_stops_at_depth_limit() {
    let (mut app, _root) = app();
    command_line::run(&mut app, "again");
    assert!(app.ui.message.contains("16"), "{}", app.ui.message);
}

#[test]
fn open_hook_loop_stops_at_depth_limit() {
    let (mut app, root) = app();
    app.open_rel("a.md");
    let message = app.ui.message.clone();
    let _ = fs::remove_dir_all(&root);
    assert!(
        message.contains("note_open") && message.contains("16"),
        "{message}"
    );
}
