use std::io::Write;

use super::InstallError;
use crate::languages_check::safe;
use crate::{LanguageSpec, Languages};

pub(crate) fn each(
    languages: &Languages,
    names: &[String],
    out: &mut dyn Write,
    mut step: impl FnMut(&LanguageSpec) -> Result<(), String>,
) -> Result<bool, InstallError> {
    let names: Vec<&str> = if names.is_empty() {
        languages
            .language
            .iter()
            .map(|spec| spec.name.as_str())
            .collect()
    } else {
        names.iter().map(String::as_str).collect()
    };
    let mut failed = 0;
    for name in &names {
        let result = languages
            .get(name)
            .ok_or_else(|| "unknown language".to_string())
            .and_then(|spec| safe(spec).map(|()| spec))
            .and_then(&mut step);
        match result {
            Ok(()) => writeln!(out, "{name} ok")?,
            Err(reason) => {
                failed += 1;
                writeln!(out, "{name} failed: {reason}")?;
            }
        }
    }
    writeln!(out, "{} ok, {failed} failed", names.len() - failed)?;
    Ok(failed == 0)
}
