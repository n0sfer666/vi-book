mod autocmd;
mod doautocmd;
mod lex;
mod listing;
mod selector;

use super::{Context, Error};
use crate::{autocmd::Bus, options::SetCommand};
use lex::Kind;
use std::cell::Cell;

struct Active<'a>(&'a Cell<usize>);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

impl Context<'_> {
    pub fn supports(input: &str) -> bool {
        lex::take(input).is_some_and(|(command, _)| command.kind != Kind::Unknown)
    }

    pub(super) fn execute_ex(&self, mut input: &str) -> Result<(), Error> {
        self.ex_depth.set(self.ex_depth.get() + 1);
        let _active = Active(&self.ex_depth);
        while let Some((command, rest)) = lex::take(input) {
            self.ex_command(&command)?;
            input = rest;
        }
        Ok(())
    }

    pub(crate) fn executing_ex(&self) -> bool {
        self.ex_depth.get() > 0
    }

    fn ex_command(&self, command: &lex::Command<'_>) -> Result<(), Error> {
        if command.bang
            && matches!(
                command.kind,
                Kind::Doautocmd | Kind::Set | Kind::Setlocal | Kind::Setglobal
            )
        {
            return Err(Error::Ex(format!("E477: No ! allowed: {}", command.raw)));
        }
        match command.kind {
            Kind::Autocmd => self.ex_autocmd(command.args, command.bang),
            Kind::Augroup => {
                let bus = self.autocmd_bus()?;
                if command.args.is_empty() {
                    if command.bang {
                        return Err(Error::Ex("E471: Argument required".into()));
                    }
                    self.messages.borrow_mut().push(
                        bus.groups()
                            .into_iter()
                            .map(|(_, name)| name)
                            .collect::<Vec<_>>()
                            .join("  "),
                    );
                } else if command.bang {
                    if bus.delete_ex_group(command.args)? {
                        self.messages
                            .borrow_mut()
                            .push("W19: Deleting augroup that is still in use".into());
                    }
                } else {
                    bus.select_group(command.args);
                }
                Ok(())
            }
            Kind::Doautocmd => self.ex_doautocmd(command.args),
            Kind::Set | Kind::Setlocal | Kind::Setglobal => {
                let name = match command.kind {
                    Kind::Setlocal => "setlocal",
                    Kind::Setglobal => "setglobal",
                    _ => "set",
                };
                let parsed = SetCommand::parse(&format!("{name} {}", command.args))
                    .ok_or_else(|| Error::Command(command.raw.into()))?;
                self.run_set(&parsed)
            }
            Kind::Enew if command.args.is_empty() => self.enew(),
            Kind::Buffer => self.select_buffer(command.args),
            Kind::File => self.name_buffer(command.args),
            Kind::Edit => self.edit_buffer(command.args, command.bang),
            Kind::Write => self.write_buffer(command.args, command.bang),
            _ => Err(Error::Command(command.raw.into())),
        }
    }

    pub(super) fn autocmd_bus(&self) -> Result<&Bus, Error> {
        self.observer
            .and_then(|observer| observer.autocmd_bus())
            .ok_or_else(|| Error::Ex("autocmd commands require an attached event bus".into()))
    }
}
