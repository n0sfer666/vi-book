use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn create(root: &Path, name: &str) -> io::Result<Option<PathBuf>> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(None);
    }
    let rel = if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    };
    let abs = root.join(&rel);
    if !abs.exists() {
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)?;
        }
        let stem = rel
            .trim_end_matches(".md")
            .rsplit('/')
            .next()
            .unwrap_or(&rel);
        fs::write(&abs, format!("# {stem}\n\n"))?;
    }
    Ok(Some(abs))
}
