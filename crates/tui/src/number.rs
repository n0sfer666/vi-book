use v_ide_view::{Grid, Rect, Style};

pub(crate) fn draw(grid: &mut Grid, area: Rect, top: usize, lines: usize, enabled: bool) -> Rect {
    if !enabled {
        return area;
    }
    let digits = lines.max(1).to_string().len();
    let width = u16::try_from((digits + 1).max(4))
        .unwrap_or(u16::MAX)
        .min(area.w.saturating_sub(1));
    let field = usize::from(width.saturating_sub(1));
    for (y, line) in (area.y..area.y.saturating_add(area.h)).zip(top..lines) {
        let label = format!("{:>field$} ", line + 1);
        grid.put_str(area.x, y, &label, Style::default().dim(), width);
    }
    Rect {
        x: area.x + width,
        w: area.w - width,
        ..area
    }
}
