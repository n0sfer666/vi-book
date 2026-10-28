use std::sync::OnceLock;

use v_ide_syntax::{Languages, set_dir, set_languages};

static READY: OnceLock<()> = OnceLock::new();

pub fn hermetic() {
    READY.get_or_init(|| {
        set_languages(Languages::builtin());
        set_dir(super::scratch(&format!(
            "hermetic-{}",
            env!("CARGO_CRATE_NAME")
        )));
    });
}
