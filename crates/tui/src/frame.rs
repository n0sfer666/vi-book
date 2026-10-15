use v_ide_view::Grid;

pub struct Frame {
    pub grid: Grid,
    pub cursor: Option<(u16, u16)>,
}

impl Frame {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            grid: Grid::new(w, h),
            cursor: None,
        }
    }
}
