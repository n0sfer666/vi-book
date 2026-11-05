use crate::app::App;
use crate::clipboard;
use v_ide_engine::mode::Mode;
use v_ide_keys::Key;

pub fn freshest<'a>(system: &'a str, last_synced: &str) -> Option<&'a str> {
    (!system.is_empty() && system != last_synced).then_some(system)
}

pub fn editor_key(app: &mut App, key: Key) {
    inject_paste(app, key);
    let buffer = app.workspace.current_id();
    let before = app.workspace.current().editor.clip_yank_version();
    crate::autocmd::editor_key(app, key);
    if let Some(doc) = app.workspace.doc(buffer)
        && doc.editor.clip_yank_version() != before
        && let Some(text) = doc.editor.yanked()
    {
        write_clipboard(app, text);
    }
}

pub fn sync_out(app: &mut App) {
    if let Some(text) = app.workspace.current().editor.yanked() {
        write_clipboard(app, text);
    }
}

fn write_clipboard(app: &mut App, text: String) {
    clipboard::set(&text);
    app.last_clip = text;
}

fn inject_paste(app: &mut App, key: Key) {
    if !matches!(key.plain_char(), Some('p' | 'P')) {
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
