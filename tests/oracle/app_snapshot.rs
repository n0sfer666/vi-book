use std::time::Instant;

use v_ide_editor::document::Document;
use v_ide_keys::{TIMEOUT, parse_keys};
use v_ide_text::Position;
use v_ide_tui::{app::App, command_line};

use crate::fixture::{Case, Snapshot};

pub fn run(case: &Case) -> Snapshot {
    let mut app = App::new(Document::new(&case.before));
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
        let mode = match map.mode {
            v_ide_keys::MapMode::Normal => "n",
            v_ide_keys::MapMode::Insert => "i",
            v_ide_keys::MapMode::Visual => "x",
            v_ide_keys::MapMode::OpPending => "o",
            v_ide_keys::MapMode::Cmdline => "c",
        };
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
    }
}
