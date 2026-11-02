#[path = "autocmd/glob_gold.rs"]
mod native;

use v_ide_editor::autocmd::{Pattern, patterns};

#[test]
fn glob_matches_neovim_0126_native_matrix() {
    let mut failures = Vec::new();
    for &(source, ignore_case, expected) in native::CASES {
        let compiled: Vec<_> = patterns(source)
            .map(|part| Pattern::compile(part).unwrap())
            .collect();
        for (index, name) in native::NAMES.iter().enumerate() {
            let actual = compiled
                .iter()
                .any(|pattern| pattern.matches(name, name, ignore_case));
            if actual != expected.contains(&index) {
                failures.push(format!(
                    "{source:?} {name:?} ignore_case={ignore_case}: {actual}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
