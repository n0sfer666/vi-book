use std::io::Write;
use std::path::Path;

use super::InstallError;
use super::report::each;
use crate::{Languages, load};

pub fn check(
    languages: &Languages,
    data: &Path,
    names: &[String],
    out: &mut dyn Write,
) -> Result<bool, InstallError> {
    writeln!(out, "{}", data.display())?;
    each(languages, names, out, |spec| {
        load(data, &spec.name)
            .map(drop)
            .map_err(|err| err.to_string())
    })
}
