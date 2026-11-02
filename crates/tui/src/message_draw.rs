use unicode_width::UnicodeWidthStr;
use v_ide_view::{Rect, Style};

use crate::{app::App, frame::Frame};

pub(crate) fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.grid.area();
    let Some(pager) = app.ui.pager.as_mut() else {
        return;
    };
    pager.resize(area.w, area.h);
    frame.cursor = None;
    if area.w == 0 || area.h == 0 {
        return;
    }
    let lines: Vec<_> = pager
        .visible()
        .take(usize::from(area.h.saturating_sub(1)))
        .collect();
    let height = u16::try_from(lines.len())
        .unwrap_or(u16::MAX)
        .saturating_add(1);
    let top = area.h.saturating_sub(height);
    frame.grid.clear(Rect {
        x: 0,
        y: top,
        w: area.w,
        h: height,
    });
    for (y, line) in (top..area.h - 1).zip(lines) {
        frame.grid.put_str(0, y, line, Style::default(), area.w);
    }
    let prompt = pager.prompt();
    frame
        .grid
        .put_str(0, area.h - 1, prompt, Style::default().reverse(), area.w);
    let col = u16::try_from(prompt.width())
        .unwrap_or(u16::MAX)
        .min(area.w - 1);
    frame.cursor = Some((col, area.h - 1));
}
