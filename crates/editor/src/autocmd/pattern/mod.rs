mod class;
mod compile;
mod machine;
mod named;
mod scan;
mod split;
mod unicode;
mod unicode_data;

pub use split::patterns;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatternError {
    #[error("E219: Missing {{")]
    MissingOpen,
    #[error("E220: Missing }}")]
    MissingClose,
    #[error("E944: Reverse range in character class")]
    ReverseRange,
    #[error("Vim-regexp insertion {0:?} is not supported in autocmd patterns")]
    UnsupportedRegex(String),
}

#[derive(Debug, Clone)]
pub struct Pattern {
    nodes: Vec<machine::Node>,
    end: usize,
    directories: bool,
}

impl Pattern {
    pub fn compile(source: &str) -> Result<Self, PatternError> {
        compile::compile(source)
    }

    pub fn matches(&self, short: &str, full: &str, ignore_case: bool) -> bool {
        if self.directories {
            self.run(short, ignore_case) || (short != full && self.run(full, ignore_case))
        } else {
            self.run(full.rsplit('/').next().unwrap_or(full), ignore_case)
        }
    }
}
