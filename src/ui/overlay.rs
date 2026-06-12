use ratatui::Frame;
use ratatui::layout::{Position as ScreenPosition, Rect};
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::ui::app::App;
use crate::ui::list_nav;

pub fn draw_panel(frame: &mut Frame, app: &App, area: Rect) {
    let Some(panel) = app.panel.as_ref() else {
        return;
    };
    let rows = panel.rows();
    let inner_height = area.height.saturating_sub(2) as usize;
    let selected = panel.selected();
    let offset = list_nav::scroll_offset(selected, inner_height);
    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .skip(offset)
        .take(inner_height)
        .map(|(i, label)| {
            if i == selected && app.panel_focus {
                Line::styled(
                    label.clone(),
                    TuiStyle::default().add_modifier(Modifier::REVERSED),
                )
            } else {
                Line::from(label.clone())
            }
        })
        .collect();
    let border = if app.panel_focus {
        TuiStyle::default().add_modifier(Modifier::BOLD)
    } else {
        TuiStyle::default().add_modifier(Modifier::DIM)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .title(panel.title());
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn draw_picker(frame: &mut Frame, app: &App, area: Rect) {
    let Some(picker) = app.picker.as_ref() else {
        return;
    };
    let width = (area.width.saturating_sub(4)).min(70);
    let height = (area.height.saturating_sub(2)).min(16);
    let rect = Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };
    let (offset, items) = picker.window(height.saturating_sub(2) as usize);
    let mut lines = vec![Line::from(format!("> {}", picker.query()))];
    for (i, item) in items.iter().enumerate() {
        let selected = offset + i == picker.selected();
        let marker = if selected { "› " } else { "  " };
        let style = if selected {
            TuiStyle::default().add_modifier(Modifier::REVERSED)
        } else {
            TuiStyle::default()
        };
        lines.push(Line::styled(format!("{marker}{item}"), style));
    }
    let block = Block::default().borders(Borders::ALL).title(" notes ");
    frame.render_widget(Paragraph::new(lines).block(block), rect);
    let cursor_col = rect.x + 3 + picker.query().chars().count() as u16;
    frame.set_cursor_position(ScreenPosition::new(cursor_col, rect.y + 1));
}
