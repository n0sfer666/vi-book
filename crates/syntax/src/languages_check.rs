use std::collections::HashSet;
use std::path::{Component, Path};

use crate::load::valid;
use crate::{LanguageSpec, Languages};

pub(crate) fn validate(languages: &Languages) -> Result<(), String> {
    let mut keys = HashSet::new();
    for spec in &languages.language {
        for key in std::iter::once(&spec.name).chain(&spec.aliases) {
            if !valid(key) {
                return Err(format!("invalid language name: {key:?}"));
            }
            if !keys.insert(key.as_str()) {
                return Err(format!("duplicate language name or alias: {key:?}"));
            }
        }
        if let Some(subpath) = &spec.grammar.subpath
            && !relative(subpath)
        {
            return Err(format!("invalid subpath: {subpath:?}"));
        }
    }
    Ok(())
}

pub(crate) fn safe(spec: &LanguageSpec) -> Result<(), String> {
    if !valid(&spec.name) {
        return Err("invalid language name".to_string());
    }
    match spec.grammar.subpath.as_deref().is_none_or(relative) {
        true => Ok(()),
        false => Err("invalid subpath".to_string()),
    }
}

fn relative(subpath: &str) -> bool {
    Path::new(subpath)
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}
