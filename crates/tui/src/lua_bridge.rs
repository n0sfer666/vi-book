use std::env;
use std::path::PathBuf;

use v_ide_view::{Color, Depth};

use crate::app::App;
use crate::base_keymap;
use crate::config_path;
use crate::lua_keymaps;
use crate::menu::MenuItem;
use crate::requests;
use crate::theme::{self, Face, Theme};
use v_ide_editor::module::Modules;
use v_ide_lua::host::PluginHost;
use v_ide_lua::module::LuaModule;
use v_ide_markdown::wrap_mode::WrapMode;

pub fn load_config(app: &mut App) {
    let resolved = config_path::resolve(|key| env::var_os(key), |path| path.exists());
    load_config_from(app, resolved);
}

pub fn load_config_from(app: &mut App, resolved: Option<(PathBuf, Option<&'static str>)>) {
    app.configure_options();
    crate::messages::batch(app, |app| load_config_inner(app, resolved));
}

fn load_config_inner(app: &mut App, mut resolved: Option<(PathBuf, Option<&'static str>)>) {
    if resolved.is_some() {
        let activated = app.modules.activate(&app.workspace, "lua");
        requests::apply(app, activated);
    }
    let Some(host) = app
        .modules
        .get::<LuaModule>()
        .and_then(LuaModule::owned_host)
    else {
        return;
    };
    let warning = resolved.as_mut().and_then(|(_, warning)| warning.take());
    let mut errors = Vec::new();
    let mut failed = false;
    let mut events = Vec::new();
    if let Some((path, _)) = resolved
        && let Ok(script) = std::fs::read_to_string(path)
    {
        let ((result, reported), deferred) = crate::core_exec::config_context(app, |cx| {
            let checkpoint = cx.diagnostic_checkpoint();
            let result = host.load_with_context(cx, &script);
            let reported = result
                .as_ref()
                .err()
                .is_some_and(|error| error.is_reported_since(checkpoint));
            (result, reported)
        });
        events = deferred;
        failed = result.is_err();
        if let Err(error) = result
            && !reported
        {
            errors.push(format!("config error: {error}"));
        }
    }
    drop((
        host.take_creates(),
        host.take_opens(),
        host.take_external_opens(),
    ));
    app.sync_timeout();
    if let Some(warning) = warning.filter(|_| app.ui.message.is_empty()) {
        app.ui.set_message(warning.to_string());
    }
    let failed = failed || !errors.is_empty();
    if let Some(error) = errors.into_iter().next_back() {
        app.ui.set_message(error);
    }
    apply_opts(app, failed);
    install_keymaps(app, failed);
    let _ = crate::core_exec::finish_events(app, events);
}

fn host(modules: &Modules) -> Option<&PluginHost> {
    modules.get::<LuaModule>().and_then(LuaModule::host)
}

fn install_keymaps(app: &mut App, failed: bool) {
    let Some(host) = host(&app.modules) else {
        return;
    };
    let mut keymap = base_keymap::build(app.insert_escape.as_deref());
    let installed = lua_keymaps::install(&mut keymap, &host.keymaps(), &app.leader);
    if let Some(error) = installed.or(host.keymap_errors().pop()).filter(|_| !failed) {
        app.ui.set_message(error);
    }
    app.keymap = keymap;
}

fn apply_opts(app: &mut App, failed: bool) {
    let Some(host) = host(&app.modules) else {
        return;
    };
    let wrap = host.opt_string("wrap").and_then(|s| WrapMode::parse(&s));
    if let Some(leader) = host.opt_string("leader") {
        let keys = v_ide_keys::parse_keys(&leader);
        if !keys.is_empty() {
            app.leader = keys;
        } else if !failed {
            app.ui.set_message("leader: empty sequence");
        }
    }
    if let Some(seq) = host.opt_string("insert_escape") {
        app.insert_escape = Some(seq).filter(|s| !s.is_empty());
    }
    if let Some(pos) = host.opt_string("menu_position") {
        app.menu_center = pos == "center";
    }
    if let Some(number) = host.opt_bool("read_number") {
        app.read_number = number;
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
    if let Some(wrap) = wrap {
        app.set_wrap(wrap);
    }
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
