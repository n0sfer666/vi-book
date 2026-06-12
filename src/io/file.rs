use std::fs;
use std::path::Path;

use crate::io::error::Error;

pub fn read(path: &Path) -> Result<String, Error> {
    Ok(fs::read_to_string(path)?)
}

pub fn write(path: &Path, text: &str) -> Result<(), Error> {
    Ok(fs::write(path, text)?)
}
