use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use ratatui::DefaultTerminal;

pub fn resolve(root: &Path, target: &str) -> Option<PathBuf> {
    let direct = root.join(target);
    if direct.is_file() {
        return Some(direct);
    }
    let name = Path::new(target).file_name()?;
    find_by_name(root, name)
}

fn find_by_name(dir: &Path, name: &std::ffi::OsStr) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let hidden = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'));
        if hidden {
            continue;
        }
        if path.is_dir() {
            dirs.push(path);
        } else if path.file_name() == Some(name) {
            return Some(path);
        }
    }
    dirs.iter().find_map(|d| find_by_name(d, name))
}

pub fn open_url(url: &str) {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    let _ = Command::new(opener).arg(url).spawn();
}

pub fn run(terminal: &mut DefaultTerminal, editor: &str, path: &Path) -> io::Result<()> {
    let mut parts = editor.split_whitespace();
    let Some(program) = parts.next() else {
        return Ok(());
    };
    let args: Vec<&str> = parts.collect();
    ratatui::restore();
    let result = Command::new(program).args(&args).arg(path).status();
    *terminal = ratatui::init();
    let _ = terminal.clear();
    result.map(|_| ())
}
