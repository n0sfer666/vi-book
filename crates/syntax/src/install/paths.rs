use std::path::{Path, PathBuf};

use crate::LanguageSpec;

pub(crate) fn source(data: &Path, spec: &LanguageSpec) -> PathBuf {
    data.join("sources").join(&spec.name)
}

pub(crate) fn grammar_root(data: &Path, spec: &LanguageSpec) -> PathBuf {
    let source = source(data, spec);
    match &spec.grammar.subpath {
        Some(subpath) => source.join(subpath),
        None => source,
    }
}
