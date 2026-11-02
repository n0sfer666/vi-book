use std::time::Instant;

use v_ide_editor::document::Document;
use v_ide_editor::options::{Access, Name, Value};
use v_ide_keys::{TIMEOUT, parse_keys};
use v_ide_text::Position;
use v_ide_tui::{app::App, command_line};

use crate::fixture::{Case, Options, Snapshot};

pub fn run(case: &Case) -> Snapshot {
    let mut app = App::new(Document::new(&case.before));
    if case.lua {
        let script = case
            .maps
            .iter()
            .map(|map| {
                format!(
                    "vibook.keymap({:?}, {}, {}, {{ remap = {}, nowait = {} }})",
                    mode_name(map.mode),
                    literal(&map.lhs),
                    literal(&map.rhs),
                    !map.noremap,
                    map.nowait
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        crate::common::load_lua(&mut app, "oracle-lua", &script);
    }
    app.workspace
        .current_mut()
        .editor
        .set_cursor(Position::new(case.cursor[0], case.cursor[1]));
    if let Some([width, height]) = case.screen {
        let mut frame = v_ide_tui::frame::Frame::new(width, height);
        v_ide_tui::render::draw(&mut frame, &mut app);
    }
    for message in &case.messages {
        app.ui.set_message(message.clone());
    }
    app.ui.message.clear();
    for map in &case.maps {
        if case.lua {
            break;
        }
        let mode = mode_name(map.mode);
        let command = if map.noremap { "noremap" } else { "map" };
        let flags = if map.nowait { "<nowait> " } else { "" };
        command_line::run(
            &mut app,
            &format!("{mode}{command} {flags}{} {}", map.lhs, map.rhs),
        );
        assert!(
            app.ui.message.is_empty(),
            "{}: {}",
            case.name,
            app.ui.message
        );
    }
    for key in parse_keys(&case.keys) {
        if app.ui.pager.is_some() {
            v_ide_tui::router::handle(&mut app, key);
        } else {
            app.typeahead.feed(key);
            app.timeout(Instant::now());
        }
    }
    let mut now = Instant::now();
    for _ in 0..1000 {
        if app.typeahead.deadline().is_none() {
            break;
        }
        now += TIMEOUT;
        app.timeout(now);
    }
    assert!(
        app.typeahead.deadline().is_none(),
        "{}: typeahead did not settle",
        case.name
    );
    if case.screen.is_some() {
        assert!(
            app.ui.pager.is_none()
                && app.ui.command.is_none()
                && app.ui.menu.is_none()
                && !app.ui.help,
            "{}: input left an overlay or command line",
            case.name
        );
    }
    let editor = &app.workspace.current().editor;
    let cursor = editor.cursor();
    Snapshot {
        text: editor.text(),
        cursor: [cursor.line, cursor.col],
        mode: super::snapshot::mode_name(editor.mode()).to_string(),
        unnamed: editor.yanked().unwrap_or_default(),
        options: case.options.then(|| options(&app)),
    }
}

fn mode_name(mode: v_ide_keys::MapMode) -> &'static str {
    match mode {
        v_ide_keys::MapMode::Normal => "n",
        v_ide_keys::MapMode::Insert => "i",
        v_ide_keys::MapMode::Visual => "x",
        v_ide_keys::MapMode::OpPending => "o",
        v_ide_keys::MapMode::Cmdline => "c",
    }
}

fn literal(text: &str) -> String {
    let mut equals = String::new();
    while text.contains(&format!("]{equals}]")) {
        equals.push('=');
    }
    format!("[{equals}[\n{text}]{equals}]")
}

fn options(app: &App) -> Options {
    let value = |name, access| app.workspace.option(name, access);
    let Value::Number(timeoutlen) = value(Name::Timeoutlen, Access::Global) else {
        panic!("timeoutlen must be numeric");
    };
    Options {
        timeoutlen,
        number: value(Name::Number, Access::Local) == Value::Bool(true),
        global_number: value(Name::Number, Access::Global) == Value::Bool(true),
        readonly: value(Name::Readonly, Access::Local) == Value::Bool(true),
        global_readonly: value(Name::Readonly, Access::Global) == Value::Bool(true),
    }
}
