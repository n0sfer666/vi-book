use v_ide_keys::{MapCommand, MapError, Mapping, format_keys};

use crate::app::App;
use crate::map_action;

pub(crate) fn execute(app: &mut App, input: &str) -> Option<bool> {
    let result = match MapCommand::parse(input, app.leader) {
        Ok(Some(command)) => apply(app, command),
        Ok(None) => return None,
        Err(error) => Err(error),
    };
    match result {
        Ok(()) => Some(true),
        Err(error) => {
            app.ui.message = error.to_string();
            Some(false)
        }
    }
}

fn apply(app: &mut App, command: MapCommand) -> Result<(), MapError> {
    match command {
        MapCommand::Map {
            modes,
            lhs,
            rhs,
            remap,
            nowait,
            unique,
        } => {
            let rhs = map_action::rhs(rhs);
            let mapping = Mapping { rhs, remap, nowait };
            let mut replaced = Vec::new();
            for mode in modes {
                let old = app
                    .keymap
                    .insert(mode, &lhs, mapping.clone())
                    .map_err(|_| MapError::InvalidArgument)?;
                let collision = unique && old.is_some();
                replaced.push((mode, old));
                if collision {
                    for (mode, old) in replaced {
                        match old {
                            Some(old) => {
                                let _ = app.keymap.insert(mode, &lhs, old);
                            }
                            None => {
                                app.keymap.remove(mode, &lhs);
                            }
                        }
                    }
                    return Err(MapError::AlreadyExists(format_keys(&lhs)));
                }
            }
        }
        MapCommand::Unmap { modes, lhs } => {
            let mut removed = false;
            for mode in modes {
                removed |= app.keymap.remove(mode, &lhs).is_some();
            }
            if !removed {
                return Err(MapError::NoSuchMapping);
            }
        }
        MapCommand::Clear { modes } => {
            for mode in modes {
                app.keymap.clear(mode);
            }
        }
        MapCommand::List { .. } => {
            app.ui.message = String::from("mapping list: not implemented yet");
        }
    }
    Ok(())
}
