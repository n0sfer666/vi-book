use super::{Context, Error};
use crate::workspace::DocId;

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
        let old = self.workspace.borrow().current_id();
        if old == id {
            return Ok(false);
        }
        self.check_target(id)?;
        if let Some(enter) = self.leaving_event("BufLeave", old, id)? {
            return Ok(enter);
        }
        if self.last_window(old)
            && let Some(enter) = self.leaving_event("BufWinLeave", old, id)?
        {
            return Ok(enter);
        }
        if self.last_window(old) {
            let events: &[&str] = if self.workspace.borrow().discardable(old) {
                &["BufDelete", "BufWipeout"]
            } else {
                &["BufHidden"]
            };
            for event in events {
                if let Some(enter) = self.leaving_event(event, old, id)? {
                    return Ok(enter);
                }
            }
        }
        self.activate_document(id);
        Ok(true)
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
