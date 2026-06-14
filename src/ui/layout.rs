use ratatui::layout::Rect;

pub const MAX_COLUMN: usize = 90;

pub fn column_width(area: Rect) -> usize {
    (area.width as usize).clamp(1, MAX_COLUMN)
}

pub fn text_column(area: Rect) -> Rect {
    let width = column_width(area) as u16;
    let left = area.x + area.width.saturating_sub(width) / 2;
    Rect {
        x: left,
        y: area.y,
        width: width.min(area.width),
        height: area.height,
    }
}
