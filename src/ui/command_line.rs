use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::app::App;

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
        "w" => save(app),
        "q" => quit(app, false),
        "q!" => quit(app, true),
        "wq" => {
            save(app);
            if app.message.is_empty() {
                app.quit = true;
            }
        }
        "" => {}
        other => app.message = format!("not an editor command: {other}"),
    }
}

fn save(app: &mut App) {
    match app.editor.save() {
        Ok(()) => {
            app.saved_text = app.editor.text();
            app.message = String::from("written");
        }
        Err(err) => app.message = format!("save failed: {err}"),
    }
}

fn quit(app: &mut App, force: bool) {
    if !force && app.editor.text() != app.saved_text {
        app.message = String::from("unsaved changes (use :q! to discard)");
    } else {
        app.quit = true;
    }
}
