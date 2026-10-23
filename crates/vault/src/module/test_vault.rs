use std::fs;
use std::path::{Path, PathBuf};

use super::batch::Batch;

pub fn vault(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/tmp")
        .join(format!("vault-unit-{name}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("sub/b.md"), "see [[a]] #idea\n").unwrap();
    root
}

pub fn rescan() -> Batch {
    Batch {
        rescan: true,
        ..Batch::default()
    }
}
