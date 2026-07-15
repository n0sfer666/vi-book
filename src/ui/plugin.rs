use std::env;
use std::path::PathBuf;

use ratatui::style::Color;

use crate::lua::host::PluginHost;
use crate::markdown::wrap_mode::WrapMode;
use crate::ui::app::App;
use crate::ui::command_line;
use crate::ui::menu::MenuItem;
use crate::ui::theme::{self, Face};
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
    let host = match PluginHost::new() {
        Ok(host) => host,
        Err(err) => {
            app.message = format!("plugin init error: {err}");
            return;
        }
    };
    for (name, source) in crate::lua::defaults::ALL {
        if let Err(err) = host.load(source) {
            app.message = format!("default plugin '{name}' error: {err}");
        }
    }
    if let Some(path) = config_path()
        && path.exists()
        && let Ok(script) = std::fs::read_to_string(&path)
        && let Err(err) = host.load(&script)
    {
        app.message = format!("config error: {err}");
    }
    host.set_vault(app.session.items(), app.session.index.tags());
    app.host = Some(host);
    apply_opts(app);
}

pub fn follow_link(app: &mut App, target: &str) -> bool {
    if app.host.is_none() {
        return false;
    }
    if let Some(Err(err)) = app
        .host
        .as_ref()
        .map(|host| host.fire_with("follow_link", target))
    {
        app.message = format!("plugin error: {err}");
        return false;
    }
    let (opens, external) = match &app.host {
        Some(host) => (host.take_opens(), host.take_external_opens()),
        None => return false,
    };
    if let Some(rel) = external.into_iter().next_back() {
        match crate::ui::external::resolve(&app.session.root, &rel) {
            Some(path) => app.external_request = Some(path),
            None => app.message = format!("file not found: {rel}"),
        }
        return true;
    }
    if let Some(rel) = opens.into_iter().next_back() {
        app.open_rel(&rel);
        return true;
    }
    false
}

pub fn apply_opts(app: &mut App) {
    let Some(host) = &app.host else {
        return;
    };
    if let Some(wrap) = host.opt_string("wrap").and_then(|s| WrapMode::parse(&s)) {
        app.wrap = wrap;
    }
    if let Some(hidden) = host.opt_bool("scan_hidden")
        && hidden != app.session.scan_hidden
    {
        app.session.scan_hidden = hidden;
        let _ = app.session.reindex();
        host.set_vault(app.session.items(), app.session.index.tags());
    }
    if let Some(leader) = host.opt_string("leader").and_then(|s| s.chars().next()) {
        app.leader = leader;
    }
    if let Some(seq) = host.opt_string("insert_escape") {
        app.insert_escape = Some(seq).filter(|s| !s.is_empty());
    }
    if let Some(pos) = host.opt_string("menu_position") {
        app.menu_center = pos == "center";
    }
    app.code_editor = host.opt_string("code_editor");
    app.menu_extra = host
        .menu_items()
        .into_iter()
        .map(|(key, label, action)| MenuItem::leaf(key.chars().next().unwrap_or(' '), label, action))
        .collect();
    apply_colors(app);
}

pub fn sync_vault_host(app: &App) {
    if let Some(host) = &app.host {
        host.set_vault(app.session.items(), app.session.index.tags());
    }
}

fn apply_colors(app: &mut App) {
    let Some(host) = &app.host else {
        return;
    };
    for spec in host.colors() {
        let Some(style) = theme::style_for_name(&spec.name) else {
            continue;
        };
        let face = Face {
            color: spec
                .fg
                .as_deref()
                .and_then(|name| name.parse::<Color>().ok()),
            bold: spec.bold,
            italic: spec.italic,
            underline: spec.underline,
            dim: spec.dim,
        };
        app.theme.set(style, face);
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
