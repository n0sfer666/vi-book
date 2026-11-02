use crate::{
    autocmd::{EventName, Names},
    workspace::{DocId, Workspace},
};

use super::Context;

impl Context<'_> {
    pub fn autocmd_names(
        &self,
        event: EventName,
        buffer: DocId,
        pattern: Option<&str>,
    ) -> Option<Names> {
        let workspace = self.workspace.borrow();
        let doc = workspace.doc(buffer)?;
        let file = match pattern {
            Some(pattern) => pattern.to_string(),
            None => match doc.path.as_deref() {
                Some(path) => full_name(&workspace, &path.to_string_lossy()),
                None => full_name(&workspace, &doc.name),
            },
        };
        let full = if event.is_file_pattern() {
            full_name(&workspace, &file)
        } else {
            file.clone()
        };
        Some(Names {
            short: file.clone(),
            file,
            full,
        })
    }
}

fn full_name(workspace: &Workspace, name: &str) -> String {
    let url = name.find("://").is_some_and(|end| {
        end > 1
            && name.as_bytes()[0].is_ascii_alphabetic()
            && name[..end]
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"+-.".contains(&byte))
    });
    if name.is_empty() || name.starts_with(['/', '~']) || url {
        return name.to_string();
    }
    let base = workspace.absolute("");
    let base = base
        .canonicalize()
        .unwrap_or_else(|_| base.components().collect());
    let (directory, tail) = name.rsplit_once('/').unwrap_or(("", name));
    if let Ok(parent) = base.join(directory).canonicalize() {
        let path = parent.join(tail);
        let path = if matches!(tail, "." | "..") {
            path.canonicalize().unwrap_or(path)
        } else {
            path
        };
        return path.to_string_lossy().into_owned();
    }
    let mut name = name;
    while let Some(rest) = name.strip_prefix("./") {
        name = rest;
    }
    format!("{}/{name}", base.to_string_lossy())
}
