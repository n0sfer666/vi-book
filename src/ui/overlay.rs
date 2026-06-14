use ratatui::Frame;
use ratatui::layout::{Position as ScreenPosition, Rect};
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::ui::app::App;
use crate::ui::help;
use crate::ui::list_nav;
use crate::ui::menu::Menu;

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
    frame.render_widget(Clear, area);
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
    frame.render_widget(Clear, rect);
    frame.render_widget(Paragraph::new(lines).block(block), rect);
    let cursor_col = rect.x + 3 + picker.query().chars().count() as u16;
    frame.set_cursor_position(ScreenPosition::new(cursor_col, rect.y + 1));
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

pub fn draw_menu(frame: &mut Frame, menu: &Menu, area: Rect) {
    let height = (menu.items.len() as u16 + 2).min(area.height.max(3));
    let rect = if menu.center {
        centered(area, 44, height)
    } else {
        Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(height),
            width: area.width,
            height,
        }
    };
    let lines: Vec<Line> = menu
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let text = format!(" [{}]  {}", item.key, item.label);
            if i == menu.selected {
                Line::styled(text, TuiStyle::default().add_modifier(Modifier::REVERSED))
            } else {
                Line::from(text)
            }
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(menu.title.clone());
    frame.render_widget(Clear, rect);
    frame.render_widget(Paragraph::new(lines).block(block), rect);
}

pub fn draw_help(frame: &mut Frame, _app: &App, area: Rect) {
    let rect = centered(area, 64, help::BINDINGS.len() as u16 + 4);
    let mut lines: Vec<Line> = help::BINDINGS
        .iter()
        .map(|(lhs, rhs)| Line::from(format!(" {lhs:<18}{rhs}")))
        .collect();
    lines.push(Line::from(""));
    lines.push(Line::styled(
        " enter: instruction   shift+enter: developers   esc: close ",
        TuiStyle::default().add_modifier(Modifier::DIM),
    ));
    let block = Block::default().borders(Borders::ALL).title(" help ");
    frame.render_widget(Clear, rect);
    frame.render_widget(Paragraph::new(lines).block(block), rect);
}
