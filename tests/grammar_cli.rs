use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[path = "../crates/syntax/tests/common/mod.rs"]
mod toy;

use toy::repo::{toy_languages, toy_repo};

const USAGE: &str = "v-ide: usage: v-ide grammar fetch|build|check [LANG...]\n";

fn command(home: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_v-ide"));
    command
        .args(args)
        .env("XDG_DATA_HOME", home.join("data"))
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env_remove("CC");
    command
}

fn v_ide(home: &Path, args: &[&str]) -> Output {
    command(home, args).output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn configure(home: &Path, toml: &str) -> std::path::PathBuf {
    let path = home.join("config/v-ide/languages.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, toml).unwrap();
    path
}

#[test]
fn grammar_fetch_build_check_end_to_end() {
    let repo = toy::scratch("cli-repo");
    let (url, rev) = toy_repo(&repo);
    let home = toy::scratch("cli-home");
    configure(&home, &toy_languages(&url, &rev));
    for sub in ["fetch", "build"] {
        let output = v_ide(&home, &["grammar", sub, "toy", "kid"]);
        assert_eq!(text(&output.stderr), "", "{sub}");
        assert_eq!(
            text(&output.stdout),
            "toy ok\nkid ok\n2 ok, 0 failed\n",
            "{sub}"
        );
        assert!(output.status.success(), "{sub}");
    }
    let output = v_ide(&home, &["grammar", "check", "toy"]);
    let data = home.join("data/v-ide");
    let expected = format!("{}\ntoy ok\n1 ok, 0 failed\n", data.display());
    assert_eq!(text(&output.stdout), expected);
    assert!(output.status.success());
}

#[test]
fn grammar_check_without_languages_covers_builtin_and_override() {
    let home = toy::scratch("cli-check-all");
    configure(&home, &toy_languages("file:///missing", "abc"));
    let output = v_ide(&home, &["grammar", "check"]);
    assert_eq!(output.status.code(), Some(1));
    let stdout = text(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], home.join("data/v-ide").display().to_string());
    assert!(lines[1].starts_with("rust failed: not found: "), "{stdout}");
    assert!(stdout.contains("\nkid failed: not found: "), "{stdout}");
    assert_eq!(lines.last(), Some(&"0 ok, 14 failed"));
}

#[test]
fn grammar_without_known_subcommand_prints_usage() {
    let home = toy::scratch("cli-usage");
    for args in [&["grammar"][..], &["grammar", "nope"][..]] {
        let output = v_ide(&home, args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert_eq!(text(&output.stderr), USAGE, "{args:?}");
        assert_eq!(text(&output.stdout), "", "{args:?}");
    }
}

#[test]
fn grammar_missing_tool_fails_before_work() {
    let home = toy::scratch("cli-tools");
    configure(&home, &toy_languages("file:///missing", "abc"));
    let output = command(&home, &["grammar", "build", "toy"])
        .env("CC", "/nonexistent/v-ide-cc")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        text(&output.stderr),
        "v-ide: /nonexistent/v-ide-cc not found in PATH\n"
    );
    assert_eq!(text(&output.stdout), "");
    let output = command(&home, &["grammar", "fetch", "toy"])
        .env("PATH", &home)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stderr), "v-ide: git not found in PATH\n");
    assert_eq!(text(&output.stdout), "");
}

#[test]
fn grammar_broken_override_fails_before_work() {
    let home = toy::scratch("cli-broken");
    let path = configure(&home, "[[language]]\nname = 1\n");
    let output = v_ide(&home, &["grammar", "check"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(
        stderr.starts_with(&format!("v-ide: {}: ", path.display())),
        "{stderr}"
    );
    assert_eq!(text(&output.stdout), "");
}

#[test]
fn grammar_build_splits_cc_and_treats_failing_probe_as_missing() {
    let repo = toy::scratch("cli-cc-repo");
    let (url, rev) = toy_repo(&repo);
    let home = toy::scratch("cli-cc-home");
    configure(&home, &toy_languages(&url, &rev));
    assert!(v_ide(&home, &["grammar", "fetch", "toy"]).status.success());
    for cc in ["cc -O0 -g", ""] {
        let output = command(&home, &["grammar", "build", "toy"])
            .env("CC", cc)
            .output()
            .unwrap();
        assert_eq!(text(&output.stderr), "", "{cc:?}");
        assert_eq!(text(&output.stdout), "toy ok\n1 ok, 0 failed\n", "{cc:?}");
    }
    let output = command(&home, &["grammar", "build", "toy"])
        .env("CC", "false")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stderr), "v-ide: false not found in PATH\n");
}

#[test]
fn grammar_fetch_ignores_inherited_git_dir() {
    let repo = toy::scratch("cli-gitdir-repo");
    let (url, rev) = toy_repo(&repo);
    let home = toy::scratch("cli-gitdir-home");
    configure(&home, &toy_languages(&url, &rev));
    let outer = home.join("outer");
    fs::create_dir_all(&outer).unwrap();
    toy::repo::git(&outer, &["init", "--quiet"]);
    let output = command(&home, &["grammar", "fetch", "toy"])
        .env("GIT_DIR", outer.join(".git"))
        .env("GIT_WORK_TREE", &outer)
        .output()
        .unwrap();
    assert_eq!(text(&output.stdout), "toy ok\n1 ok, 0 failed\n");
    toy::repo::git(&outer, &["symbolic-ref", "HEAD"]);
    assert!(
        fs::read_dir(&outer).unwrap().count() == 1,
        "outer work tree untouched"
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_arguments_do_not_panic() {
    use std::os::unix::ffi::OsStrExt;
    let home = toy::scratch("cli-non-utf8");
    let bad = std::ffi::OsStr::from_bytes(b"a\xff");
    let output = command(&home, &[])
        .arg(&home)
        .arg(bad)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert_ne!(output.status.code(), Some(101));
    assert!(!text(&output.stderr).contains("panicked"));
}
