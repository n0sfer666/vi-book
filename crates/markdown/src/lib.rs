#![forbid(unsafe_code)]

pub mod atom;
pub mod blocks;
pub mod extract;
pub mod facts;
pub mod frontmatter_scan;
#[cfg(feature = "highlight")]
mod grammar;
pub mod highlight;
pub mod inline;
pub mod link;
pub mod module;
pub mod parser;
pub mod render;
mod render_block;
pub mod rendered;
pub mod style;
pub mod syntax;
pub mod table;
pub mod target;
pub mod width;
pub mod wrap;
pub mod wrap_mode;
