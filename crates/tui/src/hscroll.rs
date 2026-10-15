use v_ide_view::{Grid, Rect, Style};

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

pub fn markers<F>(grid: &mut Grid, area: Rect, left: usize, height: usize, line_len: F)
where
    F: Fn(usize) -> Option<usize>,
{
    let width = usize::from(area.w);
    if width == 0 {
        return;
    }
    let dim = Style::default().dim();
    for row in 0..height {
        let Some(len) = line_len(row) else {
            continue;
        };
        let y = area.y + row as u16;
        if left > 0 && len > 0 {
            grid.put_str(area.x, y, LEFT.encode_utf8(&mut [0; 4]), dim, 1);
        }
        if len > left + width {
            let x = area.x + area.w - 1;
            grid.put_str(x, y, RIGHT.encode_utf8(&mut [0; 4]), dim, 1);
        }
    }
}
