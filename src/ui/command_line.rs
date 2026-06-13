use std::fs;

use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::app::App;
use crate::ui::panel::Panel;
use crate::ui::{actions, daily_cmd, plugin};

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
            run(app, &command);
        }
        _ => {}
    }
}

pub fn run(app: &mut App, command: &str) {
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
        "help" => app.help = true,
        "pick" => app.open_picker(),
        "toggle-view" => actions::toggle_view(app),
        "" => {}
        other => {
            if let Some(name) = other.strip_prefix("new ") {
                create_note(app, name);
            } else if !plugin::run_command(app, other) {
                app.message = format!("not an editor command: {other}");
            }
        }
    }
}

fn create_note(app: &mut App, name: &str) {
    let name = name.trim();
    if name.is_empty() {
        return;
    }
    let rel = if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    };
    let abs = app.session.absolute(&rel);
    if !abs.exists() {
        if let Some(parent) = abs.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            app.message = format!("create failed: {err}");
            return;
        }
        let stem = rel.trim_end_matches(".md").rsplit('/').next().unwrap_or(&rel);
        if let Err(err) = fs::write(&abs, format!("# {stem}\n\n")) {
            app.message = format!("create failed: {err}");
            return;
        }
        let _ = app.session.reindex();
    }
    app.open_rel(&rel);
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
