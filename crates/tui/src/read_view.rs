use crossterm::event::KeyEvent;

use crate::app::App;
use crate::read_select::{code_block_at, toggle_select, yank_line, yank_selection};
use crate::view::View;
use crate::{clip_sync, event_map, read_motion};
use v_ide_engine::mode::Mode;
use v_ide_keys::{Code, Key, Mods};
use v_ide_markdown::render::render_with;
use v_ide_markdown::rendered::Rendered;

pub fn cached_render(app: &mut App, width: usize) -> Rendered {
    let text = app.workspace.current().editor.text();
    let wrap = app.wrap();
    let hit = matches!(&app.ui.read.read_cache, Some((t, w, m, _)) if t == &text && *w == width && *m == wrap);
    if !hit {
        let rendered = render_with(&text, width, wrap);
        app.ui.read.read_cache = Some((text, width, wrap, rendered));
    }
    app.ui
        .read
        .read_cache
        .as_ref()
        .map(|(_, _, _, r)| r.clone())
        .unwrap_or_default()
}

pub fn enter(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    let idx = app.workspace.current().editor.text_index();
    app.ui.read.read_cursor = rendered.to_rendered(idx);
    app.ui.read.view = View::Read;
}

pub fn leave(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    if let Some(idx) = source_at(
        &rendered,
        app.ui.read.read_cursor.0,
        app.ui.read.read_cursor.1,
    ) {
        app.workspace.current_mut().editor.set_cursor_index(idx);
    }
    app.ui.read_select = None;
    app.ui.read.view = View::Edit;
}

pub fn handle_key(app: &mut App, event: KeyEvent, width: usize) {
    let Some(key) = event_map::to_stroke(event).map(|stroke| stroke.key) else {
        return;
    };
    match (key.mods(), key.code()) {
        (Mods::NONE, Code::Char('v')) => return toggle_select(app, Mode::Visual),
        (Mods::NONE, Code::Char('V')) => return toggle_select(app, Mode::VisualLine),
        (Mods::CTRL, Code::Char('v')) => return toggle_select(app, Mode::VisualBlock),
        (Mods::NONE, Code::Esc) => return app.ui.read_select = None,
        (Mods::NONE, Code::Char('y')) if app.ui.read_select.is_some() => {
            return yank_selection(app, width);
        }
        (Mods::NONE, Code::Char('Y')) => return yank_line(app, width),
        _ => {}
    }
    if delegates_to_edit(key) {
        leave(app, width);
        clip_sync::editor_key(app, key);
        return;
    }
    let rendered = cached_render(app, width);
    let last_line = rendered.lines.len().saturating_sub(1);
    let (line, col) = app.ui.read.read_cursor;
    match (key.mods(), key.code()) {
        (Mods::NONE, Code::Char('j') | Code::Down) => {
            app.ui.read.read_cursor = clamp(&rendered, line + 1, col)
        }
        (Mods::NONE, Code::Char('k') | Code::Up) => {
            app.ui.read.read_cursor = clamp(&rendered, line.saturating_sub(1), col)
        }
        (Mods::NONE, Code::Char('h') | Code::Left) => {
            app.ui.read.read_cursor = clamp(&rendered, line, col.saturating_sub(1))
        }
        (Mods::NONE, Code::Char('l') | Code::Right) => {
            app.ui.read.read_cursor = clamp(&rendered, line, col + 1)
        }
        (Mods::NONE, Code::Char('w')) => {
            app.ui.read.read_cursor = read_motion::word_forward(&rendered, (line, col))
        }
        (Mods::NONE, Code::Char('b')) => {
            app.ui.read.read_cursor = read_motion::word_back(&rendered, (line, col))
        }
        (Mods::NONE, Code::Char('e')) => {
            app.ui.read.read_cursor = read_motion::word_end(&rendered, (line, col))
        }
        (Mods::NONE, Code::Char('W')) => {
            app.ui.read.read_cursor = read_motion::element_forward(&rendered, (line, col))
        }
        (Mods::NONE, Code::Char('B')) => {
            app.ui.read.read_cursor = read_motion::element_back(&rendered, (line, col))
        }
        (Mods::NONE, Code::Char('g')) => {
            if app.ui.read.read_pending_g {
                app.ui.read.read_cursor = (0, 0);
            }
            app.ui.read.read_pending_g = !app.ui.read.read_pending_g;
            return;
        }
        (Mods::NONE, Code::Char('G')) => app.ui.read.read_cursor = clamp(&rendered, last_line, 0),
        (Mods::NONE, Code::Char('0')) if app.ui.read.read_pending_g => {
            app.ui.read.read_cursor = (line, 0)
        }
        (Mods::NONE, Code::Char('$')) if app.ui.read.read_pending_g => {
            app.ui.read.read_cursor = clamp(&rendered, line, usize::MAX)
        }
        (Mods::NONE, Code::Char('0')) => {
            app.ui.read.read_cursor = (rendered.logical(line).start, 0)
        }
        (Mods::NONE, Code::Char('$')) => {
            let last = rendered.logical(line).end - 1;
            app.ui.read.read_cursor = clamp(&rendered, last, usize::MAX)
        }
        (Mods::NONE, Code::Char('i')) => leave(app, width),
        (Mods::CTRL, Code::Char('t')) => leave(app, width),
        (Mods::NONE, Code::Enter) => {
            if let Some(idx) = source_at(&rendered, line, col) {
                app.workspace.current_mut().editor.set_cursor_index(idx);
            }
            if let Some(code) = code_block_at(
                &app.workspace.current().editor.text(),
                app.workspace.current().editor.cursor().line,
            ) {
                app.workspace.current_mut().editor.yank(code, false);
                clip_sync::sync_out(app);
                app.ui.message = String::from("copied code block to clipboard");
            } else {
                crate::link_follow::follow_at_cursor(app);
            }
        }
        _ => {}
    }
    app.ui.read.read_pending_g = false;
}

fn delegates_to_edit(key: Key) -> bool {
    matches!(key.plain_char(), Some(c) if "pPdcxoOaAIsrR".contains(c))
}

pub(crate) fn clamp(rendered: &Rendered, line: usize, col: usize) -> (usize, usize) {
    let last = rendered.lines.len().saturating_sub(1);
    let line = line.min(last);
    let len = rendered
        .lines
        .get(line)
        .map_or(0, |l| l.text().chars().count());
    (line, col.min(len.saturating_sub(1)))
}

fn source_at(rendered: &Rendered, line: usize, col: usize) -> Option<usize> {
    let len = rendered.lines.get(line)?.text().chars().count();
    (col..len)
        .chain((0..col).rev())
        .find_map(|c| rendered.to_source(line, c))
        .or(Some(0))
}
