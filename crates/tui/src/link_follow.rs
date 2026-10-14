use crate::actions;
use crate::app::App;
use crate::external;
use crate::help;
use crate::plugin;
use v_ide_vault::follow::{Follow, follow};
use v_ide_vault::link::Link;
use v_ide_vault::target::{Target, target_at};

pub fn follow_at_cursor(app: &mut App) {
    let Some(target) = target_at(
        &app.workspace.current().editor.text(),
        app.workspace.current().editor.text_index(),
    ) else {
        return;
    };
    match target {
        Target::Url(url) => app.url_request = Some(url),
        Target::Wiki(link) => follow_wiki(app, &link),
        Target::Path(path) => follow_path(app, &path),
    }
}

fn follow_wiki(app: &mut App, link: &Link) {
    if plugin::follow_link(app, &link.target) {
        return;
    }
    match follow(&app.session.index, link) {
        Follow::Open(path) => app.open_rel(&path),
        Follow::Disambiguate(paths) => {
            app.ui.picker = Some(crate::picker::Picker::new(paths, "e"));
        }
        Follow::Create(name) => {
            app.ui.message = format!("note '{name}' does not exist (press : to create)");
        }
    }
}

fn follow_path(app: &mut App, target: &str) {
    if open_bundled(app, target) {
        return;
    }
    let Some(rel) = resolve(app, target) else {
        return;
    };
    if plugin::follow_link(app, &rel) {
        return;
    }
    app.open_rel(&rel);
}

fn open_bundled(app: &mut App, target: &str) -> bool {
    if help::bundled_doc(&app.workspace.current().name).is_none() {
        return false;
    }
    match help::bundled_doc(target) {
        Some(content) => {
            actions::open_doc(app, target, content);
            true
        }
        None => false,
    }
}

fn resolve(app: &App, target: &str) -> Option<String> {
    if let Some(rel) = resolve_relative(app, target) {
        return Some(rel);
    }
    let abs = external::resolve(&app.session.root, target)?;
    abs.strip_prefix(&app.session.root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
}

fn resolve_relative(app: &App, target: &str) -> Option<String> {
    let parent = std::path::Path::new(&app.workspace.current().name).parent()?;
    let rel = parent.join(target);
    let candidate = rel.to_str()?.to_string();
    app.session
        .absolute(&candidate)
        .is_file()
        .then(|| candidate.replace('\\', "/"))
}
