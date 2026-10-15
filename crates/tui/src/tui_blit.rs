use ratatui::layout::Position;

use crate::frame::Frame;
use crate::tui_style;

pub(crate) fn blit(frame: &Frame, out: &mut ratatui::Frame) {
    let grid = &frame.grid;
    let buf = out.buffer_mut();
    for y in 0..grid.h {
        for x in 0..grid.w {
            let cell = grid.cell(x, y);
            if cell.width > 0 {
                buf[(x, y)]
                    .set_symbol(cell.symbol.as_str())
                    .set_style(tui_style::from_cell(cell.style));
            }
        }
    }
    if let Some((x, y)) = frame.cursor {
        out.set_cursor_position(Position::new(x, y));
    }
}
