#![forbid(unsafe_code)]

pub mod autocmd;
pub mod disk;
pub mod document;
pub mod execution;
pub mod filetype;
pub mod io;
pub mod module;
pub mod options;
pub mod path;
pub mod workspace;

pub use crate::io::error::Error;
