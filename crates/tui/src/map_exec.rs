use v_ide_keys::{MapCommand, MapError, Mapping, format_keys};

use crate::app::App;
use crate::map_action;

pub(crate) fn execute(app: &mut App, input: &str) -> Option<bool> {
    let result = match MapCommand::parse_with_leader(input, &app.leader) {
        Ok(Some(command)) => apply(app, command),
        Ok(None) => return None,
        Err(error) => Err(error),
    };
    match result {
        Ok(()) => Some(true),
        Err(error) => {
            app.ui.set_message(error.to_string());
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
            if unique && modes.iter().any(|&mode| app.keymap.contains(mode, &lhs)) {
                return Err(MapError::AlreadyExists(format_keys(&lhs)));
            }
            app.keymap
                .insert_modes(&modes, &lhs, mapping)
                .map_err(|_| MapError::InvalidArgument)?;
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
        MapCommand::List { modes, lhs } => {
            crate::map_listing::output(app, &modes, &lhs);
        }
    }
    Ok(())
}
