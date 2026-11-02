use std::fs;
use std::path::Path;

use crate::io::error::Error;

pub fn read(path: &Path) -> Result<String, Error> {
    Ok(fs::read_to_string(path)?)
}

pub fn write(path: &Path, text: &str) -> Result<(), Error> {
    Ok(fs::write(path, text)?)
}

pub(crate) fn readonly(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return true;
    };
    if metadata.permissions().readonly() {
        return true;
    }
    #[cfg(unix)]
    {
        rustix::fs::access(path, rustix::fs::Access::WRITE_OK).is_err()
    }
    #[cfg(not(unix))]
    {
        false
    }
}
