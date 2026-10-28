use std::fs;
use std::path::Path;

use super::paths::{grammar_root, source};
use crate::languages_check::safe;
use crate::{LanguageSpec, Languages};

pub(crate) fn highlights(
    languages: &Languages,
    data: &Path,
    spec: &LanguageSpec,
) -> Result<String, String> {
    let mut parts = Vec::new();
    collect(languages, data, spec, &mut Vec::new(), &mut parts)?;
    Ok(parts.join("\n"))
}

fn collect(
    languages: &Languages,
    data: &Path,
    spec: &LanguageSpec,
    chain: &mut Vec<String>,
    parts: &mut Vec<String>,
) -> Result<(), String> {
    if chain.contains(&spec.name) {
        return Err(format!("inherits cycle at {}", spec.name));
    }
    chain.push(spec.name.clone());
    for parent in &spec.inherits {
        let parent = languages
            .get(parent)
            .ok_or_else(|| format!("unknown parent language: {parent}"))?;
        safe(parent)?;
        collect(languages, data, parent, chain, parts)?;
    }
    chain.pop();
    parts.push(read(data, spec)?);
    Ok(())
}

fn read(data: &Path, spec: &LanguageSpec) -> Result<String, String> {
    let query = Path::new("queries").join("highlights.scm");
    let source = source(data, spec);
    let path = [grammar_root(data, spec), source.clone()]
        .into_iter()
        .map(|root| root.join(&query))
        .find(|path| path.is_file())
        .ok_or_else(|| format!("missing highlights.scm in {}", source.display()))?;
    fs::read_to_string(path).map_err(|err| err.to_string())
}
