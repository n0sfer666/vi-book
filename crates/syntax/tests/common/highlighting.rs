use std::sync::OnceLock;

use v_ide_syntax::{Languages, set_dir, set_languages};

const LANGUAGES: &str = r#"[[language]]
name = "toy"
aliases = ["ty"]

[language.grammar]
git = "file:///unused"
rev = "abc"

[[language]]
name = "ghost"

[language.grammar]
git = "file:///unused"
rev = "abc"
"#;

static READY: OnceLock<()> = OnceLock::new();

pub fn toy_highlighting() {
    READY.get_or_init(|| {
        let root = super::scratch(&format!("toy-highlighting-{}", env!("CARGO_CRATE_NAME")));
        super::install_toy(&root, "toy");
        set_languages(Languages::parse(LANGUAGES).unwrap());
        set_dir(root);
    });
}
