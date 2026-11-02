use v_ide_editor::options::{Access, Argument, Change, Error, Name, Operation, Scope, SetCommand};

use crate::app::App;

pub(crate) fn execute(app: &mut App, input: &str) -> Option<bool> {
    let command = SetCommand::parse(input)?;
    let mut output = Vec::new();
    for argument in command.arguments() {
        let result =
            argument.and_then(|argument| apply(app, command.access, argument, &mut output));
        if let Err(error) = result {
            if !output.is_empty() {
                app.ui.set_message(output.join("\n"));
            }
            app.ui.set_message(error.to_string());
            return Some(false);
        }
    }
    if !output.is_empty() {
        app.ui.set_message(output.join("\n"));
    }
    Some(true)
}

fn apply(
    app: &mut App,
    access: Access,
    argument: Argument,
    output: &mut Vec<String>,
) -> Result<(), Error> {
    match argument {
        Argument::Change(change) => {
            if app.workspace.apply_option(access, &change)?.is_none() {
                output.push(
                    change
                        .name
                        .format(app.workspace.option(change.name, access)),
                );
            }
            app.sync_timeout();
        }
        Argument::ResetAll => {
            for name in Name::ALL {
                app.workspace.apply_option(
                    access,
                    &Change {
                        name,
                        operation: Operation::Reset,
                        text: String::from("all&"),
                    },
                )?;
            }
            app.sync_timeout();
        }
        Argument::List { all } => {
            let header = match access {
                Access::Effective => "--- Options ---",
                Access::Global => "--- Global option values ---",
                Access::Local => "--- Local option values ---",
            };
            output.push(header.to_string());
            for name in Name::ALL {
                if access != Access::Effective && name.scope() == Scope::Global {
                    continue;
                }
                let value = app.workspace.option(name, access);
                if all || value != name.default_value() {
                    output.push(name.format(value));
                }
            }
        }
    }
    Ok(())
}
