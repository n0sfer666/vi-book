use crate::{
    autocmd::{Action, Filter, Selector},
    execution::{Context, Error},
};

impl Context<'_> {
    pub(super) fn ex_listing(&self, filter: &Filter) -> Result<(), Error> {
        let bus = self.autocmd_bus()?;
        let rows = bus.entries(filter);
        let mut output = String::from("--- Autocommands ---");
        let mut previous = None;
        for row in rows {
            let heading = (row.event, row.attributes.group);
            if previous != Some(heading) {
                output.push('\n');
                if let Some(group) = row
                    .attributes
                    .group
                    .and_then(|id| bus.group_display_name(id))
                {
                    output.push_str(&group);
                    output.push_str("  ");
                }
                output.push_str(row.event.as_str());
                previous = Some(heading);
            }
            let pattern = match row.selector {
                Selector::File(pattern) => pattern,
                Selector::Buffer(id) => format!("<buffer={}>", self.buffer_number(id).unwrap_or(0)),
            };
            let command = match row.action {
                Action::Command(command) => command.to_string(),
                Action::Callback(callback) => callback
                    .name()
                    .map_or_else(|| "<Lua function>".into(), |name| format!("{name}()")),
            };
            output.push_str(&format!("\n    {pattern:<10}{command}"));
        }
        self.messages.borrow_mut().push(output);
        Ok(())
    }
}
