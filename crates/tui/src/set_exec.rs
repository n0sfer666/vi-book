use v_ide_editor::options::SetCommand;

use crate::app::App;

pub(crate) fn execute(app: &mut App, input: &str) -> Option<bool> {
    SetCommand::parse(input)?;
    let (result, messages) =
        crate::core_exec::with_context(app, |cx| (cx.execute(input), cx.take_messages()));
    for message in messages {
        app.ui.set_message(message);
    }
    match result {
        Ok(()) => Some(true),
        Err(error) => {
            if !matches!(error, v_ide_editor::execution::Error::Autocmd(_)) {
                app.ui.set_message(error.to_string());
            }
            Some(false)
        }
    }
}
