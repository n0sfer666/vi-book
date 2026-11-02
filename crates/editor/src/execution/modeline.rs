mod parse;

use std::cell::Cell;

use crate::options::{Access, Argument, Error as OptionError, ErrorKind, Name, SetCommand, Value};

use super::{Context, Error};

struct Active<'a>(&'a Cell<bool>);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

impl Context<'_> {
    pub fn process_modelines(&self) {
        if self.modeline_active.get()
            || self.option(Name::Modeline, Access::Local) != Value::Bool(true)
        {
            return;
        }
        let Value::Number(scan) = self.option(Name::Modelines, Access::Global) else {
            return;
        };
        let scan = usize::try_from(scan).unwrap_or(0);
        if scan == 0 {
            return;
        }
        self.modeline_active.set(true);
        let _active = Active(&self.modeline_active);
        let length = self.workspace.borrow().current().editor.line_count();
        let head = scan.min(length);
        let tail = length.saturating_sub(scan).max(head);
        for index in (0..head).chain((tail..length).rev()) {
            if self.option(Name::Modeline, Access::Local) != Value::Bool(true) {
                break;
            }
            let line = self.workspace.borrow().current().editor.line_text(index);
            let Some(settings) = line.as_deref().and_then(parse::settings) else {
                continue;
            };
            for setting in settings {
                if let Err(error) = self.modeline_setting(&setting) {
                    self.record_error(format!("modeline line {}: {error}", index + 1));
                    return;
                }
            }
        }
    }

    fn modeline_setting(&self, setting: &str) -> Result<(), Error> {
        if setting.trim_matches([' ', '\t']).is_empty() {
            return Ok(());
        }
        let command = SetCommand::parse(&format!("setlocal {setting}"))
            .ok_or_else(|| Error::Command(setting.into()))?;
        let mut output = Vec::new();
        for argument in command.arguments() {
            let argument = argument?;
            if !matches!(argument, Argument::Change(_)) {
                return Err(OptionError::new(ErrorKind::Unknown, setting).into());
            }
            self.set_argument(Access::Local, argument, &mut output)?;
        }
        if !output.is_empty() {
            self.messages.borrow_mut().push(output.join("\n"));
        }
        Ok(())
    }
}
