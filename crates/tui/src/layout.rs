use v_ide_view::Rect;

pub fn column_width(area: Rect) -> usize {
    usize::from(area.w).max(1)
}

pub fn text_column(area: Rect) -> Rect {
    area
}
