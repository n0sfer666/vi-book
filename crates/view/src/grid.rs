use crate::cell::Cell;
use crate::rect::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub w: u16,
    pub h: u16,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            w,
            h,
            cells: vec![Cell::default(); usize::from(w) * usize::from(h)],
        }
    }

    pub fn area(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: self.w,
            h: self.h,
        }
    }

    pub fn cell(&self, x: u16, y: u16) -> &Cell {
        &self.cells[self.index(x, y)]
    }

    pub fn cell_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        let index = self.index(x, y);
        &mut self.cells[index]
    }

    pub fn row_text(&self, y: u16) -> String {
        (0..self.w)
            .map(|x| self.cell(x, y))
            .filter(|cell| cell.width > 0)
            .map(|cell| cell.symbol.as_str())
            .collect()
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        *self = Self::new(w, h);
    }

    fn index(&self, x: u16, y: u16) -> usize {
        usize::from(y) * usize::from(self.w) + usize::from(x)
    }
}
