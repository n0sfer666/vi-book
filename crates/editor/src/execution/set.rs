use crate::options::{Access, Argument, Change, Name, Operation, Scope, SetCommand};

use super::{Context, Error};

impl Context<'_> {
    pub(super) fn run_set(&self, command: &SetCommand) -> Result<(), Error> {
        let before = self.errors.get();
        let mut output = Vec::new();
        let result = command
            .arguments()
            .try_for_each(|argument| self.set_argument(command.access, argument?, &mut output));
        if !output.is_empty() {
            self.messages.borrow_mut().push(output.join("\n"));
        }
        result?;
        if before != self.errors.get() {
            return Err(Error::Autocmd(self.last_error.borrow().clone()));
        }
        Ok(())
    }

    fn set_argument(
        &self,
        access: Access,
        argument: Argument,
        output: &mut Vec<String>,
    ) -> Result<(), Error> {
        match argument {
            Argument::Change(change) => {
                if self.set_option(access, &change)?.is_none() {
                    output.push(change.name.format(self.option(change.name, access)));
                }
            }
            Argument::ResetAll => {
                let mut workspace = self.workspace.borrow_mut();
                for name in Name::ALL {
                    workspace.apply_option(
                        access,
                        &Change {
                            name,
                            operation: Operation::Reset,
                            text: "all&".into(),
                        },
                    )?;
                }
            }
            Argument::List { all } => {
                output.push(
                    match access {
                        Access::Effective => "--- Options ---",
                        Access::Global => "--- Global option values ---",
                        Access::Local => "--- Local option values ---",
                    }
                    .into(),
                );
                for name in Name::ALL {
                    if access != Access::Effective && name.scope() == Scope::Global {
                        continue;
                    }
                    let value = self.option(name, access);
                    if all || value != name.default_value() {
                        output.push(name.format(value));
                    }
                }
            }
        }
        Ok(())
    }
}
