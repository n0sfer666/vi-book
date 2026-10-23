use std::path::{Component, Path};

use v_ide_editor::path::rel_path;

use crate::scan::hidden;

pub fn scoped(root: &Path, include_hidden: bool, abs: &Path) -> Option<String> {
    let rel = abs.strip_prefix(root).ok()?;
    let inside = rel.components().next().is_some()
        && rel.components().all(|c| match c {
            Component::Normal(name) => include_hidden || !hidden(name),
            _ => false,
        });
    inside.then(|| rel_path(root, abs))
}

pub fn member(root: &Path, include_hidden: bool, abs: &Path) -> Option<String> {
    let md = abs.extension().is_some_and(|ext| ext == "md");
    scoped(root, include_hidden, abs).filter(|_| md)
}
