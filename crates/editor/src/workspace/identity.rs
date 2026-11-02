use super::{DocId, Workspace};
use std::path::{Component, Path, PathBuf};

impl Workspace {
    pub fn find_path(&self, path: &Path) -> Option<DocId> {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.cwd.join(path)
        };
        let identity = normalized(&path);
        self.documents.iter().find_map(|(&id, doc)| {
            let other = doc.path.as_ref()?;
            let other = absolute(other.to_path_buf());
            let same = normalized(&other) == identity || same_inode(&path, &other);
            same.then_some(id)
        })
    }
}

pub(super) fn absolute(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir().map_or_else(|_| path.clone(), |base| base.join(&path))
    }
}

fn normalized(path: &Path) -> PathBuf {
    let path = absolute(path.to_path_buf());
    if let Ok(path) = path.canonicalize() {
        return path;
    }
    if let Some(parent) = path.parent().and_then(|parent| parent.canonicalize().ok())
        && let Some(name) = path.file_name()
    {
        return parent.join(name);
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component),
        }
    }
    normalized
}

#[cfg(unix)]
fn same_inode(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (a.metadata(), b.metadata()) {
        (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    }
}

#[cfg(not(unix))]
fn same_inode(_a: &Path, _b: &Path) -> bool {
    false
}
