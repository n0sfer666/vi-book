use v_ide_editor::{execution::Context, options::SetCommand};

use crate::app::App;

pub(crate) fn execute(app: &mut App, input: &str) -> Option<bool> {
    SetCommand::parse(input)?;
    let (result, messages) = {
        let cx = Context::new(&mut app.workspace);
        let result = cx.execute(input);
        (result, cx.take_messages())
    };
    app.sync_timeout();
    for message in messages {
        app.ui.set_message(message);
    }
    match result {
        Ok(()) => Some(true),
        Err(error) => {
            app.ui.set_message(error.to_string());
            Some(false)
        }
    }
}
