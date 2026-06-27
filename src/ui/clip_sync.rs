use crate::ui::app::App;
use crate::ui::clipboard;
use crate::vim::key::Key;
use crate::vim::mode::Mode;

pub fn freshest<'a>(system: &'a str, last_synced: &str) -> Option<&'a str> {
    (!system.is_empty() && system != last_synced).then_some(system)
}

pub fn editor_key(app: &mut App, key: Key) {
    inject_paste(app, key);
    let before = app.editor.clip_yank_version();
    app.editor.handle_key(key);
    if app.editor.clip_yank_version() != before {
        sync_out(app);
    }
}

pub fn sync_out(app: &mut App) {
    if let Some(text) = app.editor.yanked() {
        app.last_clip = text.clone();
        clipboard::set(&text);
    }
}

fn inject_paste(app: &mut App, key: Key) {
    if !matches!(key, Key::Char('p') | Key::Char('P')) {
        return;
    }
    if app.editor.mode() != Mode::Normal || app.editor.pending_register().is_some() {
        return;
    }
    let Some(system) = clipboard::get() else {
        return;
    };
    if let Some(fresh) = freshest(&system, &app.last_clip) {
        let linewise = fresh.ends_with('\n');
        app.editor.set_unnamed(fresh.to_string(), linewise);
    }
}
