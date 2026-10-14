use std::fs;
use std::io;

use v_ide_editor::module::{Cx, Event, Manifest, Module, Registration};
use v_ide_editor::path::rel_path;

use crate::daily::{DEFAULT_TEMPLATE, daily_path, expand_template, parse_daily_path};
use crate::date::Date;

static MANIFEST: Manifest = Manifest {
    name: "notebook",
    commands: &["today", "daily-prev", "daily-next"],
    triggers: &[],
};

pub struct NotebookModule;

impl NotebookModule {
    pub fn registration() -> Registration {
        Registration {
            manifest: &MANIFEST,
            make: || Box::new(Self),
        }
    }
}

impl Module for NotebookModule {
    fn activate(&mut self, _cx: &mut Cx<'_>) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx<'_>, name: &str, args: &str) -> Result<(), String> {
        if !args.trim().is_empty() {
            return Err(format!("{name}: takes no arguments"));
        }
        let date = match name {
            "today" => Date::today(),
            "daily-prev" => base_date(cx).pred(),
            "daily-next" => base_date(cx).succ(),
            _ => return Ok(()),
        };
        open_daily(cx, date).map_err(|err| format!("daily failed: {err}"))
    }

    fn event(&mut self, _cx: &mut Cx<'_>, _ev: &Event) {}
}

fn base_date(cx: &Cx<'_>) -> Date {
    cx.path()
        .map(|path| rel_path(cx.cwd(), path))
        .as_deref()
        .and_then(parse_daily_path)
        .unwrap_or_else(Date::today)
}

fn open_daily(cx: &mut Cx<'_>, date: Date) -> io::Result<()> {
    let abs = cx.cwd().join(daily_path(date));
    if !abs.exists() {
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&abs, expand_template(DEFAULT_TEMPLATE, date))?;
        cx.emit(Event::NoteCreate(abs.clone()));
    }
    cx.open(abs);
    Ok(())
}
