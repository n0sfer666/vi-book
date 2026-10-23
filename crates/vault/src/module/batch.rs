use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Default)]
pub struct Batch {
    pub paths: BTreeSet<PathBuf>,
    pub rescan: bool,
    pub errors: Vec<String>,
}
