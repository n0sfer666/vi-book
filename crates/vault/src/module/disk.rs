use std::fs;
use std::io::ErrorKind;

use crate::index::Index;
use crate::note::Note;

use super::applied::Applied;
use super::batch::Batch;
use super::lazy::LazyIndex;
use super::level::Level;
use super::plan::Plan;
use super::reset::Reset;

pub fn apply(index: &LazyIndex, batch: &Batch) -> Applied {
    if batch.rescan {
        return match index.reset() {
            Ok(Reset::Busy(err)) => {
                index.watch().rescan_later();
                failed(err)
            }
            Ok(reset @ (Reset::Rebuilt | Reset::Forgot)) => {
                index.watch().rescanned();
                Applied {
                    changed: matches!(reset, Reset::Rebuilt),
                    errors: Vec::new(),
                }
            }
            Err(err) => {
                if index.watch().rescan_failed(&err) {
                    failed(err.to_string())
                } else {
                    Applied::default()
                }
            }
        };
    }
    if batch.paths.is_empty() {
        return Applied::default();
    }
    let (root, hidden) = (index.root(), index.scan_hidden());
    let edited = index.edit(|level| match level {
        Level::Facts(facts) => {
            let plan = Plan::new(root, hidden, &batch.paths, facts.names().paths());
            refresh(facts, plan)
        }
        Level::Names(names) => {
            let plan = Plan::new(root, hidden, &batch.paths, names.paths());
            names.apply(plan.found.keys(), &plan.gone);
            Applied::default()
        }
    });
    match edited {
        Ok(applied) => applied.unwrap_or_default(),
        Err(err) => {
            index.watch().rescan_later();
            failed(err.to_string())
        }
    }
}

fn failed(err: String) -> Applied {
    Applied {
        changed: false,
        errors: vec![err],
    }
}

fn refresh(index: &mut Index, plan: Plan) -> Applied {
    let Plan { found, mut gone } = plan;
    let mut notes = Vec::new();
    let mut errors = Vec::new();
    for (rel, abs) in found {
        match fs::read_to_string(&abs) {
            Ok(source) => notes.push(Note::from_source(&rel, &source)),
            Err(err) if err.kind() == ErrorKind::NotFound => {
                gone.insert(rel);
            }
            Err(err) => errors.push(format!("{rel}: {err}")),
        }
    }
    Applied {
        changed: index.apply(notes, &gone),
        errors,
    }
}
