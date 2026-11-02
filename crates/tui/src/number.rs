use v_ide_view::{Grid, Rect, Style};

pub(crate) fn draw(grid: &mut Grid, area: Rect, top: usize, lines: usize, enabled: bool) -> Rect {
    if !enabled {
        return area;
    }
    let width = u16::try_from(gutter_width(usize::from(area.w), lines, enabled)).unwrap_or(0);
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

fn gutter_width(total: usize, lines: usize, enabled: bool) -> usize {
    if !enabled {
        return 0;
    }
    (lines.max(1).to_string().len() + 1)
        .max(4)
        .min(total.saturating_sub(1))
}

pub(crate) fn content_width(total: usize, lines: usize, enabled: bool) -> usize {
    total
        .saturating_sub(gutter_width(total, lines, enabled))
        .max(1)
}

pub(crate) fn render(
    text: &str,
    total: usize,
    wrap: v_ide_markdown::wrap_mode::WrapMode,
    enabled: bool,
) -> v_ide_markdown::rendered::Rendered {
    let mut width = content_width(total, 0, enabled);
    loop {
        let rendered = v_ide_markdown::render::render_with(text, width, wrap);
        let next = content_width(total, rendered.lines.len(), enabled);
        if next == width {
            return rendered;
        }
        width = next;
    }
}
