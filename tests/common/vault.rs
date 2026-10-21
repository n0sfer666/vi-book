use std::path::Path;

use v_ide_editor::document::Document;
use v_ide_editor::module::{Event, Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_vault::module::VaultModule;

pub fn ws_at(root: &Path, current: Option<&str>) -> Workspace {
    let mut doc = Document::new("");
    if let Some(rel) = current {
        doc.path = Some(root.join(rel));
        doc.name = rel.to_string();
    }
    let mut ws = Workspace::new(doc);
    ws.set_cwd(root.to_path_buf());
    ws
}

pub fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

pub fn is_changed(request: &Request) -> bool {
    matches!(request, Request::Emit(Event::VaultChanged { .. }))
}

pub fn items(modules: &Modules) -> Vec<String> {
    modules
        .get::<VaultModule>()
        .and_then(|vault| vault.items().ok())
        .unwrap_or_default()
}

pub fn indexed(modules: &Modules) -> usize {
    modules.get::<VaultModule>().map_or(0, |vault| {
        vault.index().map_or(0, |index| index.notes().len())
    })
}

pub fn msg(text: &str) -> Request {
    Request::Message(String::from(text))
}
