use crate::cell::Cell;
use crate::grid::Grid;
use crate::op::Op;
use crate::style::Style;

pub fn diff(prev: &Grid, next: &Grid) -> Vec<Op> {
    let resized = prev.w != next.w || prev.h != next.h;
    let mut ops = if resized { vec![Op::Clear] } else { Vec::new() };
    let blank = Cell::default();
    let mut pen: Option<Style> = None;
    for y in 0..next.h {
        let mut cursor = None;
        for x in 0..next.w {
            let cell = next.cell(x, y);
            let old = if resized { &blank } else { prev.cell(x, y) };
            if cell == old || cell.width == 0 {
                continue;
            }
            if cursor != Some(x) {
                ops.push(Op::Move { x, y });
            }
            if pen != Some(cell.style) {
                ops.push(Op::Style(cell.style));
                pen = Some(cell.style);
            }
            push_text(&mut ops, cell.symbol.as_str());
            cursor = Some(x + u16::from(cell.width));
        }
    }
    ops
}

fn push_text(ops: &mut Vec<Op>, symbol: &str) {
    if let Some(Op::Text(text)) = ops.last_mut() {
        text.push_str(symbol);
        return;
    }
    ops.push(Op::Text(symbol.to_string()));
}
