use ratatui::layout::Rect;

pub fn column_width(area: Rect) -> usize {
    (area.width as usize).max(1)
}

pub fn text_column(area: Rect) -> Rect {
    area
}
