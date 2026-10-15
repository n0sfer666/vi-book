use std::env;
use std::mem;
use std::path::PathBuf;

use v_ide_view::{Color, Depth};

use crate::app::App;
use crate::command_line;
use crate::config_path;
use crate::menu::MenuItem;
use crate::requests;
use crate::theme::{self, Face, Theme};
use v_ide_editor::module::Modules;
use v_ide_keys::key::parse_keys;
use v_ide_lua::host::PluginHost;
use v_ide_lua::module::LuaModule;
use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_vault::module::VaultModule;

pub fn load_config(app: &mut App) {
    let resolved = config_path::resolve(|key| env::var_os(key), |path| path.exists());
    load_config_from(app, resolved);
}

pub fn load_config_from(app: &mut App, mut resolved: Option<(PathBuf, Option<&'static str>)>) {
    let Some(lua) = app.modules.get::<LuaModule>() else {
        return;
    };
    let warning = resolved.as_mut().and_then(|(_, warning)| warning.take());
    let errors = lua.load_config(resolved);
    if let Some(warning) = warning.filter(|_| app.ui.message.is_empty()) {
        app.ui.message = warning.to_string();
    }
    if let Some(error) = errors.into_iter().next_back() {
        app.ui.message = error;
    }
    apply_opts(app);
    install_keymaps(app);
}

pub fn apply_keymap(app: &mut App, rhs: &str) {
    if let Some(cmd) = rhs.strip_prefix(':') {
        command_line::run(app, cmd);
    } else {
        for key in parse_keys(rhs) {
            app.workspace.current_mut().editor.handle_key(key);
        }
    }
}

fn host(modules: &Modules) -> Option<&PluginHost> {
    modules.get::<LuaModule>().and_then(LuaModule::host)
}

fn install_keymaps(app: &mut App) {
    let Some(host) = host(&app.modules) else {
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

fn apply_opts(app: &mut App) {
    let Some(host) = host(&app.modules) else {
        return;
    };
    if let Some(wrap) = host.opt_string("wrap").and_then(|s| WrapMode::parse(&s)) {
        app.wrap = wrap;
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
    if let Some(value) = host.opt_string("color_depth") {
        app.color_depth = color_depth(&value, app.color_depth);
    }
    app.menu_extra = host
        .menu_items()
        .into_iter()
        .map(|(key, label, action)| {
            MenuItem::leaf(key.chars().next().unwrap_or(' '), label, action)
        })
        .collect();
    apply_colors(&mut app.theme, host);
    let hidden = host.opt_bool("scan_hidden");
    scan_hidden(app, hidden);
}

fn color_depth(value: &str, current: Option<Depth>) -> Option<Depth> {
    match value {
        "auto" => None,
        "16" => Some(Depth::Ansi16),
        "256" => Some(Depth::Ansi256),
        "truecolor" => Some(Depth::TrueColor),
        _ => current,
    }
}

fn scan_hidden(app: &mut App, hidden: Option<bool>) {
    let current = app.vault().map(VaultModule::scan_hidden);
    let Some(value) = hidden.filter(|value| current.is_some_and(|c| c != *value)) else {
        return;
    };
    let message = mem::take(&mut app.ui.message);
    let state = if value { "on" } else { "off" };
    requests::command(app, &format!("scan-hidden {state}"));
    if app.ui.message == format!("scan hidden folders: {state}") {
        app.ui.message = message;
    }
}

fn apply_colors(theme: &mut Theme, host: &PluginHost) {
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
        theme.set(style, face);
    }
}
