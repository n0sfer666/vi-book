#![allow(dead_code)]

pub mod feed;
pub mod scratch;
pub mod vault;
pub mod watch;

use std::fs;
use std::path::{Path, PathBuf};

use v_ide_editor::document::Document;
use v_ide_engine::Editor;
use v_ide_markdown::module::MarkdownModule;
use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_text::Position;
use v_ide_tui::app::App;
use v_ide_tui::frame::Frame;
use v_ide_tui::{command_line, requests};

pub use scratch::{scratch, scratch_path};

pub struct Case {
    pub name: &'static str,
    pub before: &'static str,
    pub cursor: (usize, usize),
    pub keys: &'static str,
    pub after: &'static str,
    pub end_cursor: (usize, usize),
}

pub fn t(
    name: &'static str,
    before: &'static str,
    cursor: (usize, usize),
    keys: &'static str,
    after: &'static str,
    end_cursor: (usize, usize),
) -> Case {
    Case {
        name,
        before,
        cursor,
        keys,
        after,
        end_cursor,
    }
}

pub fn editor_at(text: &str, line: usize, col: usize) -> Editor {
    let mut editor = Editor::new(text);
    editor.set_cursor(Position::new(line, col));
    editor
}

pub fn check(cases: &[Case]) {
    let mut failures = Vec::new();
    for case in cases {
        let mut editor = editor_at(case.before, case.cursor.0, case.cursor.1);
        editor.handle_keys(case.keys);
        let text = editor.text();
        let cursor = editor.cursor();
        let want = Position::new(case.end_cursor.0, case.end_cursor.1);
        if text != case.after || cursor != want {
            failures.push(format!(
                "{}: keys {:?}\n  text got  {:?}\n  text want {:?}\n  cursor got ({}, {}), want ({}, {})",
                case.name,
                case.keys,
                text,
                case.after,
                cursor.line,
                cursor.col,
                want.line,
                want.col
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

pub fn named_doc(text: &str, name: &str) -> Document {
    let mut doc = Document::new(text);
    doc.name = name.to_string();
    doc
}

pub fn open_vault(app: &mut App, root: &Path) {
    app.workspace.set_cwd(root.to_path_buf());
    requests::startup(app);
}

pub fn temp_root(label: &str) -> PathBuf {
    scratch(&format!("lua-{label}"))
}

pub fn config_file(label: &str, script: &str) -> PathBuf {
    let path = scratch_path(&format!("init-{label}.lua"));
    fs::write(&path, script).unwrap();
    path
}

pub fn load_lua(app: &mut App, label: &str, script: &str) {
    let path = config_file(label, script);
    v_ide_tui::lua_bridge::load_config_from(app, Some((path, None)));
}

pub fn lua_app(doc: Document, label: &str, script: &str) -> App {
    let mut app = App::new(doc);
    open_vault(&mut app, &temp_root(label));
    load_lua(&mut app, label, script);
    app
}

pub fn draw(app: &mut App, w: u16, h: u16) -> Frame {
    let mut frame = Frame::new(w, h);
    v_ide_tui::render::draw(&mut frame, app);
    frame
}

pub fn wrap(app: &App) -> Option<WrapMode> {
    app.modules
        .get::<MarkdownModule>()
        .map(MarkdownModule::wrap)
}

pub fn set_wrap(app: &mut App, mode: &str) {
    command_line::run(app, &format!("wrap {mode}"));
}
