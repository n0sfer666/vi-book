use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::app::App;
use crate::ui::panel::Panel;
use crate::ui::{daily_cmd, plugin};

pub fn handle(app: &mut App, event: KeyEvent) {
    let Some(command) = app.command.as_mut() else {
        return;
    };
    match event.code {
        KeyCode::Char(c) => command.push(c),
        KeyCode::Backspace => {
            if command.pop().is_none() {
                app.command = None;
            }
        }
        KeyCode::Esc => app.command = None,
        KeyCode::Enter => {
            let command = app.command.take().unwrap_or_default();
            execute(app, &command);
        }
        _ => {}
    }
}

fn execute(app: &mut App, command: &str) {
    match command {
        "w" => {
            save(app);
        }
        "q" => quit(app, false),
        "q!" => quit(app, true),
        "wq" => {
            if save(app) {
                app.quit = true;
            }
        }
        "today" => daily_cmd::open_today(app),
        "daily-prev" => daily_cmd::open_prev(app),
        "daily-next" => daily_cmd::open_next(app),
        "tree" => toggle_panel(app, PanelKind::Tree),
        "backlinks" => toggle_panel(app, PanelKind::Backlinks),
        "" => {}
        other => {
            if !plugin::run_command(app, other) {
                app.message = format!("not an editor command: {other}");
            }
        }
    }
}

enum PanelKind {
    Tree,
    Backlinks,
}

fn toggle_panel(app: &mut App, kind: PanelKind) {
    let same = matches!(
        (&app.panel, &kind),
        (Some(Panel::Tree { .. }), PanelKind::Tree)
            | (Some(Panel::Backlinks { .. }), PanelKind::Backlinks)
    );
    if same {
        app.panel = None;
        app.panel_focus = false;
        return;
    }
    app.panel = Some(match kind {
        PanelKind::Tree => Panel::tree(&app.session.items()),
        PanelKind::Backlinks => Panel::backlinks(
            app.session
                .index
                .backlinks(&app.path)
                .iter()
                .map(|n| n.path.clone())
                .collect(),
        ),
    });
    app.panel_focus = true;
}

fn save(app: &mut App) -> bool {
    match app.editor.save() {
        Ok(()) => {
            app.saved_text = app.editor.text();
            app.message = String::from("written");
            plugin::fire(app, "note_save");
            true
        }
        Err(err) => {
            app.message = format!("save failed: {err}");
            false
        }
    }
}

fn quit(app: &mut App, force: bool) {
    if !force && app.editor.text() != app.saved_text {
        app.message = String::from("unsaved changes (use :q! to discard)");
    } else {
        app.quit = true;
    }
}
