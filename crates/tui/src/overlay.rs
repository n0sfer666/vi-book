use v_ide_view::{Grid, Rect, Style};

use crate::app::App;
use crate::frame::Frame;
use crate::help;
use crate::list_nav;
use crate::menu::Menu;
use crate::render::Line;

fn modal(grid: &mut Grid, rect: Rect, title: &str, border: Style, lines: &[Line]) {
    let inner_w = rect.w.saturating_sub(2);
    grid.clear(rect);
    grid.border(rect, "", border);
    grid.put_str(rect.x + 1, rect.y, title, Style::default(), inner_w);
    let rows = usize::from(rect.h.saturating_sub(2));
    for (y, line) in (rect.y + 1..).zip(lines.iter().take(rows)) {
        let mut x = rect.x + 1;
        for (text, style) in line {
            x += grid.put_str(x, y, text, *style, (rect.x + 1 + inner_w).saturating_sub(x));
        }
    }
}

fn plain(text: String) -> Line {
    vec![(text, Style::default())]
}

fn marked(text: String, on: bool) -> Line {
    let style = if on {
        Style::default().reverse()
    } else {
        Style::default()
    };
    vec![(text, style)]
}

pub fn draw_panel(grid: &mut Grid, app: &App, area: Rect) {
    let Some(panel) = app.ui.panel.as_ref() else {
        return;
    };
    let rows = panel.rows();
    let inner_height = usize::from(area.h.saturating_sub(2));
    let selected = panel.selected();
    let offset = list_nav::scroll_offset(selected, inner_height);
    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .skip(offset)
        .take(inner_height)
        .map(|(i, label)| marked(label.clone(), i == selected && app.ui.panel_focus))
        .collect();
    let border = if app.ui.panel_focus {
        Style::default().bold()
    } else {
        Style::default().dim()
    };
    modal(grid, area, &panel.title(), border, &lines);
}

pub fn draw_picker(frame: &mut Frame, app: &App, area: Rect) {
    let Some(picker) = app.ui.picker.as_ref() else {
        return;
    };
    let w = (area.w.saturating_sub(4)).min(70);
    let h = (area.h.saturating_sub(2)).min(16);
    let rect = Rect {
        x: area.x + (area.w.saturating_sub(w)) / 2,
        y: area.y + (area.h.saturating_sub(h)) / 2,
        w,
        h,
    };
    let (offset, items) = picker.window(usize::from(h.saturating_sub(2)));
    let mut lines = vec![plain(format!("> {}", picker.query()))];
    for (i, item) in items.iter().enumerate() {
        let selected = offset + i == picker.selected();
        let marker = if selected { "› " } else { "  " };
        lines.push(marked(format!("{marker}{item}"), selected));
    }
    modal(&mut frame.grid, rect, " notes ", Style::default(), &lines);
    let cursor_col = rect.x + 3 + picker.query().chars().count() as u16;
    frame.cursor = Some((cursor_col, rect.y + 1));
}

pub fn draw_buffer_list(frame: &mut Frame, app: &App, area: Rect) {
    let Some(list) = app.ui.buffer_list.as_ref() else {
        return;
    };
    let w = (area.w.saturating_sub(4)).min(70);
    let h = (area.h.saturating_sub(2)).min(16);
    let rect = Rect {
        x: area.x + (area.w.saturating_sub(w)) / 2,
        y: area.y + (area.h.saturating_sub(h)) / 2,
        w,
        h,
    };
    let (offset, items) = list.window(usize::from(h.saturating_sub(2)));
    let mut lines = vec![plain(format!("> {}", list.query()))];
    for (i, item) in items.iter().enumerate() {
        lines.push(marked(item.label(), offset + i == list.selected()));
    }
    modal(&mut frame.grid, rect, " buffers ", Style::default(), &lines);
    let cursor_col = rect.x + 3 + list.query().chars().count() as u16;
    frame.cursor = Some((cursor_col, rect.y + 1));
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.w);
    let h = h.min(area.h);
    Rect {
        x: area.x + (area.w.saturating_sub(w)) / 2,
        y: area.y + (area.h.saturating_sub(h)) / 2,
        w,
        h,
    }
}

pub fn draw_menu(grid: &mut Grid, menu: &Menu, area: Rect) {
    let h = (menu.items.len() as u16 + 2).min(area.h.max(3));
    let rect = if menu.center {
        centered(area, 44, h)
    } else {
        Rect {
            y: area.y + area.h.saturating_sub(h),
            h,
            ..area
        }
    };
    let lines: Vec<Line> = menu
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let key = if item.key == ' ' {
                '\u{2423}'
            } else {
                item.key
            };
            marked(format!(" [{key}]  {}", item.label), i == menu.selected)
        })
        .collect();
    modal(grid, rect, &menu.title, Style::default(), &lines);
}

pub fn draw_help(grid: &mut Grid, _app: &App, area: Rect) {
    let rect = centered(area, 64, help::BINDINGS.len() as u16 + 4);
    let mut lines: Vec<Line> = help::BINDINGS
        .iter()
        .map(|(lhs, rhs)| plain(format!(" {lhs:<18}{rhs}")))
        .collect();
    lines.push(plain(String::new()));
    lines.push(vec![(
        String::from(" enter: instruction   shift+enter: developers   esc: close "),
        Style::default().dim(),
    )]);
    modal(grid, rect, " help ", Style::default(), &lines);
}
