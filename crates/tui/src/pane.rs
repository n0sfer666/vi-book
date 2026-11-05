use crate::{app::App, autocmd, core_exec};
use v_ide_editor::{
    execution::Error,
    workspace::{DocId, Window, WindowId},
};

const MAX_PANES: usize = 3;
const UNSAVED: &str = "unsaved changes (write with :w first)";

impl App {
    pub fn pane_count(&self) -> usize {
        self.workspace.windows().len()
    }

    pub fn vsplit(&mut self, target: Option<&str>) {
        if self.pane_count() >= MAX_PANES {
            self.ui.set_message(format!("max {MAX_PANES} columns"));
            return;
        }
        let window = self.workspace.active_window();
        if !autocmd::current(self, "WinLeave") || self.workspace.active_window() != window {
            return;
        }
        self.ui.read.left = self.ui.left;
        let view = self.ui.read.fork();
        self.stash_window();
        let _ = self.workspace.split(self.workspace.current_buffer());
        self.ui.read = view;
        self.ui.left = self.ui.read.left;
        if autocmd::current(self, "WinNew")
            && autocmd::current(self, "WinEnter")
            && let Some(target) = target
        {
            self.split_file(&normalize(target));
        }
    }

    pub fn focus_next(&mut self) {
        let windows = self.workspace.windows();
        if windows.len() <= 1 {
            return;
        }
        let target = windows[(self.workspace.active_index() + 1) % windows.len()]
            .buf
            .id;
        let old = self.workspace.current_id();
        if !self.leave_window(target) {
            return;
        }
        self.stash_window();
        self.workspace.focus_next();
        self.restore_window();
        self.enter_window(old);
    }

    pub fn only(&mut self) {
        let active = self.workspace.active_window();
        let windows = self.workspace.windows();
        if windows
            .iter()
            .any(|window| window.id != active && window.buf.doc.is_dirty())
        {
            self.ui.set_message(String::from(UNSAVED));
            return;
        }
        let closing: Vec<_> = windows
            .iter()
            .rev()
            .filter(|window| window.id != active)
            .map(|window| window.id)
            .collect();
        let mut closed = Vec::new();
        let mut deliver_events = true;
        for window in closing {
            let Some((window, delivered)) = self.close_pane(window, deliver_events) else {
                break;
            };
            closed.push(window);
            deliver_events = delivered;
        }
        // Park buffers in reverse order and retain options from the last original window.
        closed.reverse();
        self.workspace.remember_closed_options(&closed);
    }

    pub fn close_active(&mut self) {
        let windows = self.workspace.windows();
        if windows.len() <= 1 {
            return;
        }
        let active = self.workspace.active_index();
        let target = if active + 1 < windows.len() {
            active + 1
        } else {
            active - 1
        };
        let window = windows[active].id;
        let old = windows[active].buf.id;
        let target = windows[target].buf.id;
        if !self.clean_window(window) || !self.leave_window(target) {
            return;
        }
        if let Some((_, delivered)) = self.close_pane(window, true) {
            self.restore_window();
            if delivered {
                self.enter_window(old);
            }
        }
    }

    fn leave_window(&mut self, target: DocId) -> bool {
        let window = self.workspace.active_window();
        (self.workspace.current_id() == target || autocmd::current(self, "BufLeave"))
            && window == self.workspace.active_window()
            && autocmd::current(self, "WinLeave")
            && window == self.workspace.active_window()
    }

    fn enter_window(&mut self, old: DocId) {
        let changed_buffer = old != self.workspace.current_id();
        if autocmd::current(self, "WinEnter") && changed_buffer {
            autocmd::current(self, "BufEnter");
        }
    }

    fn clean_window(&mut self, id: WindowId) -> bool {
        if self
            .workspace
            .windows()
            .iter()
            .any(|window| window.id == id && window.buf.doc.is_dirty())
        {
            self.ui.set_message(String::from(UNSAVED));
            return false;
        }
        true
    }

    fn close_pane(&mut self, window: WindowId, deliver_events: bool) -> Option<(Window, bool)> {
        if !self.clean_window(window) {
            return None;
        }
        let delivered =
            deliver_events && core_exec::with_context(self, |cx| cx.closing_window(window)).is_ok();
        if !self.clean_window(window) {
            return None;
        }
        let closed = self.workspace.close_window_by_id(window)?;
        self.window_states.remove(&window);
        self.read_states.retain(|(id, _), _| *id != window);
        Some((closed, delivered))
    }

    fn split_file(&mut self, rel: &str) {
        let path = self.workspace.absolute(rel);
        let existing = self.workspace.find_path(&path);
        if existing.is_none() && !path.is_file() {
            return;
        }
        let result = core_exec::with_context(self, |cx| match existing {
            Some(id) => cx.select_document(id),
            None => cx.open_existing_path(rel, false),
        });
        if let Err(error) = result
            && !matches!(error, Error::Autocmd(_))
        {
            self.ui.set_message(error.to_string());
        }
    }

    fn stash_window(&mut self) {
        self.ui.read.left = self.ui.left;
        let state = std::mem::take(&mut self.ui.read);
        self.window_states
            .insert(self.workspace.active_window(), state);
    }

    fn restore_window(&mut self) {
        let state = self
            .window_states
            .remove(&self.workspace.active_window())
            .unwrap_or_default();
        self.ui.read = state;
        self.ui.left = self.ui.read.left;
    }
}

fn normalize(name: &str) -> String {
    let name = name.trim();
    if name.contains('.') {
        name.to_string()
    } else {
        format!("{name}.md")
    }
}
