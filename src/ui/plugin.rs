use std::env;
use std::path::PathBuf;

use crate::lua::host::PluginHost;
use crate::ui::app::App;
use crate::ui::command_line;
use crate::vim::key::parse_keys;

pub fn install_keymaps(app: &mut App) {
    let Some(host) = &app.host else {
        return;
    };
    app.normal_maps = host
        .keymaps()
        .into_iter()
        .filter(|m| m.mode == "n")
        .filter_map(|m| match parse_keys(&m.lhs)[..] {
            [key] => Some((key, m.rhs)),
            _ => None,
        })
        .collect();
}

pub fn apply_keymap(app: &mut App, rhs: &str) {
    if let Some(cmd) = rhs.strip_prefix(':') {
        command_line::run(app, cmd);
    } else {
        for key in parse_keys(rhs) {
            app.editor.handle_key(key);
        }
    }
}

pub fn load_config(app: &mut App) {
    let Some(path) = config_path() else {
        return;
    };
    if !path.exists() {
        return;
    }
    let Ok(script) = std::fs::read_to_string(&path) else {
        return;
    };
    match PluginHost::new().and_then(|host| host.load(&script).map(|()| host)) {
        Ok(host) => {
            host.set_vault(app.session.items(), app.session.index.tags());
            app.host = Some(host);
        }
        Err(err) => app.message = format!("config error: {err}"),
    }
}

fn config_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("VIBOOK_CONFIG") {
        return Some(PathBuf::from(path));
    }
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/vi-book/init.lua"))
}

pub fn fire(app: &mut App, event: &str) {
    if app.host.is_none() || app.firing {
        return;
    }
    app.firing = true;
    sync_in(app);
    let result = app.host.as_ref().map(|host| host.fire(event));
    if let Some(Err(err)) = result {
        app.message = format!("plugin error: {err}");
    } else {
        sync_out(app, false);
    }
    app.firing = false;
}

pub fn run_command(app: &mut App, name: &str) -> bool {
    let Some(host) = &app.host else {
        return false;
    };
    host.set_buffer(&app.editor.text(), app.editor.text_index());
    match host.run_command(name) {
        Ok(false) => false,
        Ok(true) => {
            sync_out(app, true);
            true
        }
        Err(err) => {
            app.message = format!("plugin error: {err}");
            true
        }
    }
}

fn sync_in(app: &App) {
    if let Some(host) = &app.host {
        host.set_buffer(&app.editor.text(), app.editor.text_index());
    }
}

fn sync_out(app: &mut App, apply_opens: bool) {
    let Some(host) = &app.host else {
        return;
    };
    let new_text = host.buffer_text();
    let opens = if apply_opens {
        host.take_opens()
    } else {
        Vec::new()
    };
    if new_text != app.editor.text() {
        app.editor.replace_all(&new_text);
    }
    if let Some(rel) = opens.into_iter().next_back() {
        app.open_rel(&rel);
    }
}
