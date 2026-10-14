use crossterm::event::KeyEvent;

use crate::app::App;
use crate::read_select::{code_block_at, toggle_select, yank_line, yank_selection};
use crate::view::View;
use crate::{clip_sync, event_map, read_motion};
use v_ide_engine::mode::Mode;
use v_ide_keys::key::Key;
use v_ide_markdown::render::render_with;
use v_ide_markdown::rendered::Rendered;

pub fn cached_render(app: &mut App, width: usize) -> Rendered {
    let text = app.workspace.current().editor.text();
    let wrap = app.wrap;
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
    let Some(key) = event_map::to_key(event) else {
        return;
    };
    match key {
        Key::Char('v') => return toggle_select(app, Mode::Visual),
        Key::Char('V') => return toggle_select(app, Mode::VisualLine),
        Key::Ctrl('v') => return toggle_select(app, Mode::VisualBlock),
        Key::Esc => return app.ui.read_select = None,
        Key::Char('y') if app.ui.read_select.is_some() => return yank_selection(app, width),
        Key::Char('Y') => return yank_line(app, width),
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
    match key {
        Key::Char('j') | Key::Down => app.ui.read.read_cursor = clamp(&rendered, line + 1, col),
        Key::Char('k') | Key::Up => {
            app.ui.read.read_cursor = clamp(&rendered, line.saturating_sub(1), col)
        }
        Key::Char('h') | Key::Left => {
            app.ui.read.read_cursor = clamp(&rendered, line, col.saturating_sub(1))
        }
        Key::Char('l') | Key::Right => app.ui.read.read_cursor = clamp(&rendered, line, col + 1),
        Key::Char('w') => {
            app.ui.read.read_cursor = read_motion::word_forward(&rendered, (line, col))
        }
        Key::Char('b') => app.ui.read.read_cursor = read_motion::word_back(&rendered, (line, col)),
        Key::Char('e') => app.ui.read.read_cursor = read_motion::word_end(&rendered, (line, col)),
        Key::Char('W') => {
            app.ui.read.read_cursor = read_motion::element_forward(&rendered, (line, col))
        }
        Key::Char('B') => {
            app.ui.read.read_cursor = read_motion::element_back(&rendered, (line, col))
        }
        Key::Char('g') => {
            if app.ui.read.read_pending_g {
                app.ui.read.read_cursor = (0, 0);
            }
            app.ui.read.read_pending_g = !app.ui.read.read_pending_g;
            return;
        }
        Key::Char('G') => app.ui.read.read_cursor = clamp(&rendered, last_line, 0),
        Key::Char('0') => app.ui.read.read_cursor = (line, 0),
        Key::Char('$') => app.ui.read.read_cursor = clamp(&rendered, line, usize::MAX),
        Key::Char('i') => leave(app, width),
        Key::Ctrl('t') => leave(app, width),
        Key::Enter => {
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
    matches!(key, Key::Char(c) if "pPdcxoOaAIsrR".contains(c))
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
