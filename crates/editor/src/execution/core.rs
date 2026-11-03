use super::{Context, Error};
use crate::{
    document::Document,
    options::{Access, Name, Scope},
};

impl Context<'_> {
    pub fn text(&self) -> String {
        self.workspace.borrow().current().editor.text()
    }
    pub fn cursor(&self) -> usize {
        self.workspace.borrow().current().editor.text_index()
    }
    pub fn set_text(&self, text: &str) {
        self.workspace
            .borrow_mut()
            .current_mut()
            .editor
            .replace_all(text);
    }

    pub(super) fn enew(&self) -> Result<(), Error> {
        let mut workspace = self.workspace.borrow_mut();
        let old = workspace.current();
        let mut doc = Document::new("");
        for name in Name::ALL {
            if name.scope() == Scope::Buffer && name != Name::Readonly {
                doc.options
                    .set(name, workspace.option(name, Access::Global));
            }
        }
        if old.path.is_none()
            && old.name.is_empty()
            && old.editor.text().is_empty()
            && !old.is_dirty()
        {
            workspace.current_mut().options = doc.options;
            return Ok(());
        }
        let id = workspace.add(doc);
        workspace.select(id);
        Ok(())
    }

    pub(super) fn select_buffer(&self, input: &str) -> Result<(), Error> {
        let number = input
            .parse::<u64>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| Error::Ex(format!("E86: Buffer {input} does not exist")))?;
        let id = self
            .buffer(number)
            .ok_or_else(|| Error::Ex(format!("E86: Buffer {input} does not exist")))?;
        let removed = {
            let mut workspace = self.workspace.borrow_mut();
            let old = workspace.current_id();
            workspace.select(id);
            workspace.doc(old).is_none().then_some(old)
        };
        if let Some(old) = removed
            && let Ok(bus) = self.autocmd_bus()
        {
            bus.forget_buffer(old);
        }
        Ok(())
    }

    pub(super) fn name_buffer(&self, input: &str) -> Result<(), Error> {
        if input.is_empty() {
            return Err(Error::Ex("file without a name is not supported yet".into()));
        }
        let mut text = String::new();
        let mut escape = false;
        for ch in input.chars() {
            if escape {
                text.push(ch);
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else {
                text.push(ch);
            }
        }
        if escape {
            text.push('\\');
        }
        let mut workspace = self.workspace.borrow_mut();
        let path = workspace.absolute(&text);
        if workspace
            .find_path(&path)
            .is_some_and(|id| id != workspace.current_id())
        {
            return Err(Error::Ex(
                "E95: Buffer with this name already exists".into(),
            ));
        }
        let doc = workspace.current_mut();
        doc.rename_target(text, path);
        Ok(())
    }
}
