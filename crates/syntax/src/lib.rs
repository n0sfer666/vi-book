#![deny(unsafe_code)]

mod dir;
mod error;
mod load;
mod registry;

use std::sync::Arc;

use tree_sitter::Language;

pub use dir::{data_dir, default_dir};
pub use error::LoadError;
pub use load::load;
pub use registry::{generation, grammar, set_dir};

#[derive(Clone, Debug)]
pub struct Grammar {
    pub language: Language,
    pub highlights: Arc<str>,
}
