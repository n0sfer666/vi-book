use crate::app::App;
use crate::clipboard;
use v_ide_engine::mode::Mode;
use v_ide_keys::key::Key;

pub fn freshest<'a>(system: &'a str, last_synced: &str) -> Option<&'a str> {
    (!system.is_empty() && system != last_synced).then_some(system)
}

pub fn editor_key(app: &mut App, key: Key) {
    inject_paste(app, key);
    let before = app.workspace.current().editor.clip_yank_version();
    app.workspace.current_mut().editor.handle_key(key);
    if app.workspace.current().editor.clip_yank_version() != before {
        sync_out(app);
    }
}

pub fn sync_out(app: &mut App) {
    if let Some(text) = app.workspace.current().editor.yanked() {
        app.last_clip = text.clone();
        clipboard::set(&text);
    }
}

fn inject_paste(app: &mut App, key: Key) {
    if !matches!(key, Key::Char('p') | Key::Char('P')) {
        return;
    }
    if app.workspace.current().editor.mode() != Mode::Normal
        || app.workspace.current().editor.pending_register().is_some()
    {
        return;
    }
    let Some(system) = clipboard::get() else {
        return;
    };
    if let Some(fresh) = freshest(&system, &app.last_clip) {
        let linewise = fresh.ends_with('\n');
        app.workspace
            .current_mut()
            .editor
            .set_unnamed(fresh.to_string(), linewise);
    }
}
