use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    modified: Option<SystemTime>,
    len: u64,
}

impl Stamp {
    pub fn of(path: &Path) -> io::Result<Self> {
        let meta = fs::metadata(path)?;
        Ok(Self {
            modified: meta.modified().ok(),
            len: meta.len(),
        })
    }
}
