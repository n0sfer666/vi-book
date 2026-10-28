#![deny(unsafe_code)]

mod dir;
mod error;
mod grammar_source;
pub mod install;
mod language_spec;
mod languages;
mod languages_check;
mod languages_error;
mod load;
mod registry;

use std::sync::Arc;

use tree_sitter::Language;

pub use dir::{config_dir, data_dir, default_config_dir, default_dir};
pub use error::LoadError;
pub use grammar_source::GrammarSource;
pub use language_spec::LanguageSpec;
pub use languages::Languages;
pub use languages_error::LanguagesError;
pub use load::load;
pub use registry::{canonical, generation, grammar, set_dir, set_languages};

#[derive(Clone, Debug)]
pub struct Grammar {
    pub language: Language,
    pub highlights: Arc<str>,
}
