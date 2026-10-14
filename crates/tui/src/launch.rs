use std::io;
use std::path::{Path, PathBuf};

use v_ide_vault::scan::rel_path;

pub fn target(arg: Option<&str>) -> io::Result<(PathBuf, Option<String>)> {
    match arg {
        Some(arg) if Path::new(arg).is_dir() => Ok((PathBuf::from(arg), None)),
        Some(arg) => {
            let path = Path::new(arg);
            let root = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
            let rel = rel_path(&root, path);
            Ok((root, Some(rel)))
        }
        None => Ok((std::env::current_dir()?, None)),
    }
}
