use std::io;
use std::path::{Path, PathBuf};

use crate::vault::index::Index;
use crate::vault::scan::scan;

pub struct Session {
    pub root: PathBuf,
    pub index: Index,
}

impl Session {
    pub fn in_memory() -> Self {
        Self {
            root: PathBuf::from("."),
            index: Index::from_notes(Vec::new()),
        }
    }

    pub fn open(arg: Option<&str>) -> io::Result<(Self, Option<String>)> {
        let (root, file) = match arg {
            Some(arg) if Path::new(arg).is_dir() => (PathBuf::from(arg), None),
            Some(arg) => {
                let path = PathBuf::from(arg);
                let root = path.parent().filter(|p| !p.as_os_str().is_empty());
                (
                    root.map(Path::to_path_buf)
                        .unwrap_or_else(|| PathBuf::from(".")),
                    Some(path),
                )
            }
            None => (std::env::current_dir()?, None),
        };
        let index = Index::from_notes(scan(&root)?);
        let session = Self { root, index };
        let rel = file.as_deref().map(|f| session.relativize(f));
        Ok((session, rel))
    }

    pub fn reindex(&mut self) -> io::Result<()> {
        self.index = Index::from_notes(scan(&self.root)?);
        Ok(())
    }

    pub fn items(&self) -> Vec<String> {
        self.index.notes().iter().map(|n| n.path.clone()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.index.notes().is_empty()
    }

    pub fn absolute(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn relativize(&self, file: &Path) -> String {
        file.strip_prefix(&self.root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/")
    }
}
