use crate::app::App;
use v_ide_editor::options::{Access, Change, Name, Operation, Value};

impl App {
    pub(crate) fn configure_options(&mut self) {
        if self.defaults_applied {
            return;
        }
        self.defaults_applied = true;
        let _ = self.workspace.apply_option(
            Access::Effective,
            &Change {
                name: Name::Number,
                operation: Operation::Set(Value::Bool(true)),
                text: String::from("number"),
            },
        );
    }

    pub(crate) fn sync_timeout(&mut self) {
        if let Value::Number(value) = self.workspace.option(Name::Timeoutlen, Access::Global)
            && let Ok(millis) = u64::try_from(value)
        {
            self.typeahead
                .set_timeout(std::time::Duration::from_millis(millis));
        }
    }
}
