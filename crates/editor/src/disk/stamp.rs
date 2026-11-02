use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    modified: Option<SystemTime>,
    len: u64,
    #[cfg(unix)]
    mode: u32,
    #[cfg(not(unix))]
    readonly: bool,
}

impl Stamp {
    pub fn of(path: &Path) -> io::Result<Self> {
        let meta = fs::metadata(path)?;
        Ok(Self {
            modified: meta.modified().ok(),
            len: meta.len(),
            #[cfg(unix)]
            mode: {
                use std::os::unix::fs::MetadataExt;
                meta.mode()
            },
            #[cfg(not(unix))]
            readonly: meta.permissions().readonly(),
        })
    }

    pub(crate) fn same_mode(self, other: Self) -> bool {
        #[cfg(unix)]
        {
            self.mode == other.mode
        }
        #[cfg(not(unix))]
        {
            self.readonly == other.readonly
        }
    }
}
