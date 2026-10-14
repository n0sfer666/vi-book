use crate::app::App;
use crate::clip_sync;
use crate::read_view::cached_render;
use v_ide_engine::mode::Mode;
use v_ide_engine::visual;
use v_ide_markdown::blocks::{self, BlockKind};
use v_ide_markdown::rendered::Rendered;
use v_ide_text::position::Position;

pub(crate) fn toggle_select(app: &mut App, mode: Mode) {
    app.ui.read_select = match app.ui.read_select {
        Some((current, _)) if current == mode => None,
        Some((_, anchor)) => Some((mode, anchor)),
        None => Some((mode, app.ui.read.read_cursor)),
    };
}

pub(crate) fn yank_selection(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    if let Some((mode, anchor)) = app.ui.read_select {
        let text = selected_text(&rendered, mode, anchor, app.ui.read.read_cursor);
        app.workspace
            .current_mut()
            .editor
            .yank(text, mode == Mode::VisualLine);
        clip_sync::sync_out(app);
        app.ui.read_select = None;
        app.ui.message = String::from("copied to clipboard");
    }
}

pub(crate) fn yank_line(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    if let Some(line) = rendered.lines.get(app.ui.read.read_cursor.0) {
        let text = line.text();
        app.workspace
            .current_mut()
            .editor
            .yank(format!("{text}\n"), true);
        clip_sync::sync_out(app);
        app.ui.message = String::from("copied line to clipboard");
    }
}

pub(crate) fn code_block_at(text: &str, src_line: usize) -> Option<String> {
    blocks::analyze(text).into_iter().find_map(|block| {
        if let BlockKind::Code { lines, .. } = block.kind
            && src_line >= block.start_line
            && src_line <= block.end_line
        {
            let code = lines
                .iter()
                .map(|line| line.iter().map(|atom| atom.ch).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            return Some(code);
        }
        None
    })
}

fn selected_text(
    rendered: &Rendered,
    mode: Mode,
    anchor: (usize, usize),
    cursor: (usize, usize),
) -> String {
    let top = anchor.0.min(cursor.0);
    let bottom = anchor.0.max(cursor.0);
    (top..=bottom)
        .filter_map(|line| {
            let chars: Vec<char> = rendered.lines.get(line)?.text().chars().collect();
            let range = visual::selection_cols(
                mode,
                Position::new(anchor.0, anchor.1),
                Position::new(cursor.0, cursor.1),
                line,
                chars.len(),
            )?;
            let from = range.start.min(chars.len());
            let to = range.end.min(chars.len());
            Some(chars[from..to].iter().collect::<String>())
        })
        .collect::<Vec<_>>()
        .join("\n")
}
