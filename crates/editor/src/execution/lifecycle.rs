use super::{Context, Error};
use crate::{
    document::Document,
    workspace::{DocId, WindowId},
};

impl Context<'_> {
    pub fn select_document(&self, id: DocId) -> Result<(), Error> {
        if self.leave_for(id)? {
            self.enter_document(id)?;
        }
        Ok(())
    }

    pub(super) fn enter_document(&self, id: DocId) -> Result<(), Error> {
        self.emit_event("BufEnter", id, None)?;
        let current = self.workspace.borrow().current_id();
        self.emit_event("BufWinEnter", current, None)
    }

    pub(super) fn enter_read_document(&self, id: DocId, startup: bool) -> Result<(), Error> {
        if !startup {
            self.emit_event("BufEnter", id, None)?;
        }
        if self.buffer(0) == Some(id) {
            self.process_modelines();
        }
        let current = self.workspace.borrow().current_id();
        self.emit_event("BufWinEnter", current, None)
    }

    pub(super) fn leave_for(&self, id: DocId) -> Result<bool, Error> {
        self.leave_for_transition(id, false).map(|(enter, _)| enter)
    }

    fn leave_for_transition(
        &self,
        id: DocId,
        closing: bool,
    ) -> Result<(bool, Option<DocId>), Error> {
        let old = self.workspace.borrow().current_id();
        if old == id {
            return Ok((false, None));
        }
        self.check_target(id)?;
        if let Some(enter) = self.leaving_event("BufLeave", old, id)? {
            return Ok((enter, None));
        }
        if self.last_window(old)
            && let Some(enter) = self.leaving_event("BufWinLeave", old, id)?
        {
            return Ok((enter, None));
        }
        let mut unloaded = None;
        if self.last_window(old) {
            if closing && !self.workspace.borrow().current().is_dirty() {
                self.emit_io("BufUnload", old)?;
                unloaded = Some(old);
                if let Some(enter) = self.after_leaving(old, id)? {
                    return Ok((enter, unloaded));
                }
            } else {
                let events: &[&str] = if self.workspace.borrow().discardable(old) {
                    &["BufDelete", "BufWipeout"]
                } else {
                    &["BufHidden"]
                };
                for event in events {
                    if let Some(enter) = self.leaving_event(event, old, id)? {
                        return Ok((enter, None));
                    }
                }
            }
        }
        if unloaded.is_some() {
            self.workspace.borrow_mut().select_service(id);
        } else {
            self.activate_document(id);
        }
        Ok((true, unloaded))
    }

    pub fn closing_window(&self, window: WindowId) -> Result<(), Error> {
        let Some(buffer) = self.workspace.borrow().window_buffer(window) else {
            return Ok(());
        };
        self.emit_event("WinClosed", buffer, Some(&window.number().to_string()))?;
        let buffer = self.workspace.borrow().window_buffer(window);
        if let Some(buffer) = buffer
            && self.last_window(buffer)
        {
            self.emit_io("BufWinLeave", buffer)?;
            let remaining = self.workspace.borrow().window_buffer(window);
            if remaining == Some(buffer) && self.last_window(buffer) {
                self.emit_io("BufHidden", buffer)?;
            }
        }
        Ok(())
    }

    pub fn close_document(&self, id: DocId) -> Result<(), Error> {
        self.check_clean(id)?;
        let mut unloaded = None;
        if self.buffer(0) == Some(id) {
            let replacement = self.workspace.borrow().pool().first().map(|buf| buf.id);
            let replacement = if let Some(replacement) = replacement {
                replacement
            } else {
                let replacement = self.workspace.borrow_mut().add(Document::new(""));
                self.new_document(replacement)?;
                replacement
            };
            let (enter, did_unload) = self.leave_for_transition(replacement, true)?;
            unloaded = did_unload;
            if enter {
                self.enter_document(replacement)?;
            }
        }
        let events = ["BufUnload", "BufDelete", "BufWipeout"];
        for event in &events[usize::from(unloaded == Some(id))..] {
            if self.hidden_index(id).is_none() {
                return Ok(());
            }
            self.check_clean(id)?;
            self.emit_io(event, id)?;
        }
        self.check_clean(id)?;
        if let Some(index) = self.hidden_index(id) {
            let _ = self.workspace.borrow_mut().remove(index);
            if self.buffer_number(id).is_none()
                && let Ok(bus) = self.autocmd_bus()
            {
                bus.forget_buffer(id);
            }
        }
        Ok(())
    }

    fn check_clean(&self, id: DocId) -> Result<(), Error> {
        if self
            .workspace
            .borrow()
            .doc(id)
            .is_some_and(Document::is_dirty)
        {
            return Err(Error::Ex(
                "unsaved changes (write with :w or force-close)".into(),
            ));
        }
        Ok(())
    }

    fn hidden_index(&self, id: DocId) -> Option<usize> {
        self.workspace
            .borrow()
            .pool()
            .iter()
            .position(|buf| buf.id == id)
    }

    fn last_window(&self, id: DocId) -> bool {
        self.workspace
            .borrow()
            .windows()
            .iter()
            .filter(|window| window.buf.id == id)
            .count()
            == 1
    }

    fn check_target(&self, id: DocId) -> Result<(), Error> {
        if self.buffer_number(id).is_none() {
            return Err(Error::Ex("E201: Autocommands changed buffer".into()));
        }
        Ok(())
    }

    fn leaving_event(&self, event: &str, old: DocId, target: DocId) -> Result<Option<bool>, Error> {
        self.emit_event(event, old, None)?;
        self.after_leaving(old, target)
    }

    fn after_leaving(&self, old: DocId, target: DocId) -> Result<Option<bool>, Error> {
        self.check_target(target)?;
        let current = self.workspace.borrow().current_id();
        if current == target {
            return Ok(Some(false));
        }
        if current != old {
            self.activate_document(target);
            return Ok(Some(true));
        }
        Ok(None)
    }

    pub(super) fn activate_document(&self, target: DocId) {
        let old = self.workspace.borrow().current_id();
        self.workspace.borrow_mut().select(target);
        if self.buffer_number(old).is_none()
            && let Ok(bus) = self.autocmd_bus()
        {
            bus.forget_buffer(old);
        }
    }

    pub(super) fn new_document(&self, id: DocId) -> Result<(), Error> {
        self.emit_event("BufNew", id, None)?;
        self.emit_event("BufAdd", id, None)
    }
}
