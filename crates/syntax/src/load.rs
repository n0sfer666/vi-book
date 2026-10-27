use std::env::consts::DLL_EXTENSION;
use std::fs;
use std::io::ErrorKind;
use std::mem;
use std::ops::RangeInclusive;
use std::path::Path;
use std::sync::Arc;

use libloading::Library;
use tree_sitter::{LANGUAGE_VERSION, Language, MIN_COMPATIBLE_LANGUAGE_VERSION, Query};
use tree_sitter_language::LanguageFn;

use crate::{Grammar, LoadError};

const ABI: RangeInclusive<usize> = MIN_COMPATIBLE_LANGUAGE_VERSION..=LANGUAGE_VERSION;

type LanguageCtor = unsafe extern "C" fn() -> *const ();

pub fn load(dir: &Path, lang: &str) -> Result<Grammar, LoadError> {
    if !valid(lang) {
        return Err(LoadError::BadName(lang.to_string()));
    }
    let library = dir.join("grammars").join(format!("{lang}.{DLL_EXTENSION}"));
    if !library.is_file() {
        return Err(LoadError::NotFound(library));
    }
    let query = dir.join("queries").join(lang).join("highlights.scm");
    let highlights = fs::read_to_string(&query).map_err(|err| match err.kind() {
        ErrorKind::NotFound => LoadError::NotFound(query),
        _ => LoadError::Query(err.to_string()),
    })?;
    let language = open(&library, &format!("tree_sitter_{}", lang.replace('-', "_")))?;
    let abi = language.abi_version();
    if !ABI.contains(&abi) {
        mem::forget(language);
        return Err(LoadError::Abi(abi));
    }
    Query::new(&language, &highlights).map_err(|err| LoadError::Query(err.to_string()))?;
    Ok(Grammar {
        language,
        highlights: Arc::from(highlights),
    })
}

fn valid(lang: &str) -> bool {
    !lang.is_empty()
        && lang
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

#[allow(unsafe_code)]
fn open(path: &Path, symbol: &str) -> Result<Language, LoadError> {
    let library = unsafe { Library::new(path) }.map_err(|err| LoadError::Open(err.to_string()))?;
    let function = unsafe { library.get::<LanguageCtor>(symbol) }
        .map(|function| *function)
        .map_err(|_| LoadError::Symbol(symbol.to_string()))?;
    if unsafe { function() }.is_null() {
        return Err(LoadError::Symbol(symbol.to_string()));
    }
    mem::forget(library);
    Ok(Language::new(unsafe { LanguageFn::from_raw(function) }))
}
