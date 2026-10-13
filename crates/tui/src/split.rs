use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::layout::{column_width, text_column};
use crate::pane::Pane;
use crate::read_view::read_lines;
use crate::render::styled_line;
use crate::theme::Theme;
use crate::view::View;
use v_ide_markdown::highlight::highlight;
use v_ide_markdown::render::render_with;
use v_ide_markdown::wrap_mode::WrapMode;

pub fn split_rects(area: Rect, n: usize) -> Vec<Rect> {
    let n = n.max(1) as u16;
    if n == 1 {
        return vec![area];
    }
    let gaps = n - 1;
    let content = area.width.saturating_sub(gaps);
    let base = content / n;
    let extra = content % n;
    let mut rects = Vec::new();
    let mut x = area.x;
    for i in 0..n {
        let width = base + u16::from(i < extra);
        rects.push(Rect {
            x,
            y: area.y,
            width,
            height: area.height,
        });
        x += width + 1;
    }
    rects
}

pub fn draw_divider(frame: &mut Frame, area: Rect, x: u16) {
    if x >= area.x + area.width {
        return;
    }
    let cell = Rect {
        x,
        y: area.y,
        width: 1,
        height: area.height,
    };
    let bar: Vec<Line> = (0..area.height).map(|_| Line::from("\u{2502}")).collect();
    frame.render_widget(
        Paragraph::new(bar).style(TuiStyle::default().add_modifier(Modifier::DIM)),
        cell,
    );
}

pub fn draw_pane(frame: &mut Frame, theme: &Theme, pane: &Pane, area: Rect, wrap: WrapMode) {
    let height = area.height as usize;
    let lines = match pane.state.view {
        View::Read => {
            let rendered = render_with(&pane.buf.doc.editor.text(), column_width(area), wrap);
            read_lines(&rendered, pane.state.top, height, theme, None)
        }
        View::Edit => {
            let text = pane.buf.doc.editor.text();
            let highlights = highlight(&text);
            text.split('\n')
                .enumerate()
                .skip(pane.state.top)
                .take(height)
                .map(|(idx, line)| {
                    styled_line(
                        line,
                        highlights.get(idx).map_or(&[][..], Vec::as_slice),
                        theme,
                        None,
                    )
                })
                .collect()
        }
    };
    frame.render_widget(Paragraph::new(lines), text_column(area));
}
