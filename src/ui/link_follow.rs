use crate::ui::actions;
use crate::ui::app::App;
use crate::ui::external;
use crate::ui::help;
use crate::ui::plugin;
use crate::vault::follow::{follow, Follow};
use crate::vault::link::Link;
use crate::vault::target::{target_at, Target};

pub fn follow_at_cursor(app: &mut App) {
    let Some(target) = target_at(&app.editor.text(), app.editor.text_index()) else {
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
            app.picker = Some(crate::ui::picker::Picker::new(paths));
        }
        Follow::Create(name) => {
            app.message = format!("note '{name}' does not exist (press : to create)");
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
    if help::bundled_doc(&app.path).is_none() {
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
    let parent = std::path::Path::new(&app.path).parent()?;
    let rel = parent.join(target);
    let candidate = rel.to_str()?.to_string();
    app.session
        .absolute(&candidate)
        .is_file()
        .then(|| candidate.replace('\\', "/"))
}
