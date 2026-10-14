use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::cell::Cell;
use crate::grid::Grid;
use crate::rect::Rect;
use crate::style::Style;
use crate::symbol::Symbol;

impl Grid {
    pub fn put_str(&mut self, x: u16, y: u16, text: &str, style: Style, max_w: u16) -> u16 {
        if y >= self.h || x >= self.w {
            return 0;
        }
        let limit = self.w.min(x.saturating_add(max_w));
        let mut col = x;
        for grapheme in text.graphemes(true) {
            let width = u8::try_from(grapheme.width()).unwrap_or(u8::MAX);
            if width == 0 {
                continue;
            }
            if col.saturating_add(u16::from(width)) > limit {
                break;
            }
            self.place(
                col,
                y,
                Cell {
                    symbol: Symbol::new(grapheme),
                    width,
                    style,
                },
            );
            col += u16::from(width);
        }
        col - x
    }

    pub fn fill(&mut self, rect: Rect, style: Style) {
        for y in rect.y..rect.y.saturating_add(rect.h).min(self.h) {
            for x in rect.x..rect.x.saturating_add(rect.w).min(self.w) {
                self.place(x, y, Cell::blank(style));
            }
        }
    }

    pub fn clear(&mut self, rect: Rect) {
        self.fill(rect, Style::default());
    }

    pub fn border(&mut self, rect: Rect, title: &str, style: Style) {
        if rect.w < 2 || rect.h < 2 {
            return;
        }
        let right = rect.x + rect.w - 1;
        let bottom = rect.y + rect.h - 1;
        for x in rect.x + 1..right {
            self.set(x, rect.y, "─", style);
            self.set(x, bottom, "─", style);
        }
        for y in rect.y + 1..bottom {
            self.set(rect.x, y, "│", style);
            self.set(right, y, "│", style);
        }
        self.set(rect.x, rect.y, "┌", style);
        self.set(right, rect.y, "┐", style);
        self.set(rect.x, bottom, "└", style);
        self.set(right, bottom, "┘", style);
        self.put_str(rect.x + 1, rect.y, title, style, rect.w - 2);
    }

    fn set(&mut self, x: u16, y: u16, symbol: &str, style: Style) {
        if x < self.w && y < self.h {
            self.place(
                x,
                y,
                Cell {
                    symbol: Symbol::new(symbol),
                    width: 1,
                    style,
                },
            );
        }
    }

    fn place(&mut self, x: u16, y: u16, cell: Cell) {
        self.release(x, y);
        if cell.width == 2 {
            self.release(x + 1, y);
        }
        *self.cell_mut(x, y) = cell;
        if cell.width == 2 {
            *self.cell_mut(x + 1, y) = Cell::continuation(cell.style);
        }
    }

    fn release(&mut self, x: u16, y: u16) {
        let cell = *self.cell(x, y);
        if cell.width == 0 && x > 0 {
            let style = self.cell(x - 1, y).style;
            *self.cell_mut(x - 1, y) = Cell::blank(style);
        }
        if cell.width == 2 && x + 1 < self.w {
            *self.cell_mut(x + 1, y) = Cell::blank(cell.style);
        }
    }
}
