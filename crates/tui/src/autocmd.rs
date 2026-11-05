mod input;

use crate::app::App;
use v_ide_editor::workspace::Workspace;

pub(crate) use input::{editor_key, flush};

pub(crate) struct State {
    input: input::Input,
    pub booting: bool,
    started: bool,
}

impl State {
    pub fn new(workspace: &Workspace) -> Self {
        Self {
            input: input::Input::new(workspace),
            booting: false,
            started: false,
        }
    }
}

impl App {
    pub fn begin_startup(&mut self) {
        if !self.autocmd.started {
            self.autocmd.booting = true;
        }
    }

    pub fn finish_startup(&mut self) {
        if self.autocmd.started {
            return;
        }
        self.autocmd.started = true;
        let entered = self.autocmd.booting;
        self.autocmd.booting = true;
        if entered {
            current(self, "BufEnter");
        }
        current(self, "VimEnter");
        self.autocmd.booting = false;
    }
}

pub(crate) fn current(app: &mut App, event: &str) -> bool {
    crate::core_exec::with_context(app, |cx| {
        let Some(id) = cx.buffer(0) else {
            return Ok(());
        };
        cx.emit_event(event, id, None)
    })
    .is_ok()
}

pub(crate) fn quit(app: &mut App, force: bool) -> bool {
    if app.quit {
        return true;
    }
    let delivered = current(app, "QuitPre") && current(app, "ExitPre");
    if !force && app.any_dirty() {
        app.ui
            .set_message(String::from("unsaved changes (use :q! to discard)"));
        return false;
    }
    app.quit = true;
    if delivered && current(app, "VimLeavePre") {
        current(app, "VimLeave");
    }
    true
}
