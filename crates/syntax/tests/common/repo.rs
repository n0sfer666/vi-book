use std::fs;
use std::path::Path;
use std::process::Command;

use super::{HIGHLIGHTS, PARSER_C, PARSER_H};

pub fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .args([
            "-c",
            "user.name=toy",
            "-c",
            "user.email=toy@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

pub fn toy_repo(root: &Path) -> (String, String) {
    write(&root.join("src/parser.c"), PARSER_C);
    write(&root.join("src/tree_sitter/parser.h"), PARSER_H);
    write(&root.join("queries/highlights.scm"), HIGHLIGHTS);
    let kid = PARSER_C.replace("tree_sitter_toy", "tree_sitter_kid");
    write(&root.join("kid/src/parser.c"), &kid);
    write(&root.join("kid/src/tree_sitter/parser.h"), PARSER_H);
    git(root, &["init", "--quiet"]);
    git(root, &["add", "."]);
    git(root, &["commit", "--quiet", "-m", "toy"]);
    let rev = git(root, &["rev-parse", "HEAD"]);
    write(&root.join("queries/highlights.scm"), "(broken @x\n");
    git(root, &["commit", "--quiet", "-am", "later"]);
    (format!("file://{}", root.display()), rev)
}

pub fn toy_languages(url: &str, rev: &str) -> String {
    format!(
        r#"[[language]]
name = "toy"
aliases = ["ty"]

[language.grammar]
git = "{url}"
rev = "{rev}"

[[language]]
name = "kid"
inherits = ["toy"]

[language.grammar]
git = "{url}"
rev = "{rev}"
subpath = "kid"
"#
    )
}
