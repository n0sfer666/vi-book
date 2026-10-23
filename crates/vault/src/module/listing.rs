use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::kind::Kind;

#[derive(Default)]
pub struct Listing {
    dirs: HashMap<PathBuf, Option<HashMap<OsString, bool>>>,
}

impl Listing {
    pub fn kind(&mut self, abs: &Path) -> Option<Kind> {
        let (parent, name) = (abs.parent()?, abs.file_name()?);
        let entries = self
            .dirs
            .entry(parent.to_path_buf())
            .or_insert_with(|| read(parent).ok());
        Some(match entries.as_ref()?.get(name) {
            None => Kind::Missing,
            Some(true) => Kind::Dir,
            Some(false) => Kind::File,
        })
    }
}

fn read(dir: &Path) -> io::Result<HashMap<OsString, bool>> {
    match fs::read_dir(dir) {
        Ok(entries) => entries
            .map(|entry| {
                let entry = entry?;
                Ok((entry.file_name(), entry.file_type()?.is_dir()))
            })
            .collect(),
        Err(err) if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            Ok(HashMap::new())
        }
        Err(err) => Err(err),
    }
}
