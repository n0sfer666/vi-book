use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::Frame;

pub const LEFT: char = '\u{2039}';
pub const RIGHT: char = '\u{203a}';

pub fn scroll(left: usize, col: usize, width: usize) -> usize {
    let width = width.max(1);
    if col < left {
        col
    } else if col >= left + width {
        col + 1 - width
    } else {
        left
    }
}

pub fn markers<F>(frame: &mut Frame, area: Rect, left: usize, height: usize, line_len: F)
where
    F: Fn(usize) -> Option<usize>,
{
    let width = area.width as usize;
    if width == 0 {
        return;
    }
    let dim = TuiStyle::default().add_modifier(Modifier::DIM);
    let buf = frame.buffer_mut();
    for row in 0..height {
        let Some(len) = line_len(row) else {
            continue;
        };
        let y = area.y + row as u16;
        if left > 0 && len > 0 {
            buf[(area.x, y)].set_char(LEFT).set_style(dim);
        }
        if len > left + width {
            buf[(area.x + area.width - 1, y)]
                .set_char(RIGHT)
                .set_style(dim);
        }
    }
}
