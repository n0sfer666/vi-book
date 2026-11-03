use super::Bus;
use crate::autocmd::{Error, GroupId};

impl Bus {
    pub fn current_group(&self) -> Option<GroupId> {
        self.current_group.get()
    }

    pub fn select_group(&self, name: &str) {
        self.current_group.set(if name == "END" {
            None
        } else {
            Some(self.create_group(name, false))
        });
    }

    pub fn delete_ex_group(&self, name: &str) -> Result<bool, Error> {
        let id = self
            .group(name)
            .ok_or_else(|| Error::GroupName(name.into()))?;
        if self.current_group.get() == Some(id) {
            return Err(Error::CurrentGroup);
        }
        let mut state = self.state.borrow_mut();
        let used = state
            .registry
            .rows
            .iter()
            .any(|row| row.attributes.group == Some(id));
        state
            .registry
            .groups
            .retain(|(candidate, _)| *candidate != id);
        if used {
            state.retired.insert(id);
        }
        Ok(used)
    }

    pub fn group_display_name(&self, id: GroupId) -> Option<String> {
        let state = self.state.borrow();
        state
            .registry
            .group_name(id)
            .map(str::to_string)
            .or_else(|| state.retired.contains(&id).then(|| "--Deleted--".into()))
    }
}
