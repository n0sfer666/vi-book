use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use v_ide_editor::path::rel_path;

use crate::scan::walk;

use super::kind::Kind;
use super::listing::Listing;
use super::scope::scoped;

pub struct Plan {
    pub found: BTreeMap<String, PathBuf>,
    pub gone: BTreeSet<String>,
}

impl Plan {
    pub fn new(root: &Path, hidden: bool, paths: &BTreeSet<PathBuf>, known: &[String]) -> Self {
        let mut listing = Listing::default();
        let mut found = BTreeMap::new();
        let mut gone = BTreeSet::new();
        for abs in paths {
            let Some(rel) = scoped(root, hidden, abs) else {
                continue;
            };
            let Some((kind, seen)) = seen(root, &mut listing, hidden, abs, &rel) else {
                continue;
            };
            gone.extend(
                known_at(known, &rel)
                    .filter(|path| !seen.contains_key(*path))
                    .cloned(),
            );
            found.extend(
                seen.into_iter()
                    .filter(|(path, _)| kind != Kind::Dir || known.binary_search(path).is_err()),
            );
        }
        gone.retain(|path| !found.contains_key(path));
        Self { found, gone }
    }
}

fn seen(
    root: &Path,
    listing: &mut Listing,
    hidden: bool,
    abs: &Path,
    rel: &str,
) -> Option<(Kind, BTreeMap<String, PathBuf>)> {
    let kind = listing.kind(abs)?;
    let seen = match kind {
        Kind::Dir => {
            let mut files = Vec::new();
            walk(abs, hidden, &mut files).ok()?;
            files
                .into_iter()
                .map(|file| (rel_path(root, &file), file))
                .collect()
        }
        Kind::File if abs.extension().is_some_and(|ext| ext == "md") => {
            BTreeMap::from([(rel.to_string(), abs.to_path_buf())])
        }
        Kind::File | Kind::Missing => BTreeMap::new(),
    };
    Some((kind, seen))
}

fn known_at<'a>(known: &'a [String], rel: &'a str) -> impl Iterator<Item = &'a String> {
    let start = known.partition_point(|path| path.as_str() < rel);
    known[start..]
        .iter()
        .take_while(move |path| path.starts_with(rel))
        .filter(move |path| {
            path.strip_prefix(rel)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
        })
}
