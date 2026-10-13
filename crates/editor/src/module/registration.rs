use crate::module::{Manifest, Module};

#[derive(Clone, Copy)]
pub struct Registration {
    pub manifest: &'static Manifest,
    pub make: fn() -> Box<dyn Module>,
}
