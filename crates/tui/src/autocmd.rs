use crate::app::App;

#[derive(Default)]
pub(crate) struct State {
    pub booting: bool,
    started: bool,
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
