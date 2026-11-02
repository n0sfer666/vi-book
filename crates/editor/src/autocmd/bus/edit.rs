use std::collections::HashSet;

use crate::{
    autocmd::{Action, AutocmdId, Error, Filter, GroupId, Record, Registry},
    workspace::DocId,
};

use super::Bus;

impl Bus {
    pub fn entries(&self, filter: &Filter) -> Vec<Record<Action>> {
        self.state
            .borrow()
            .registry
            .entries(filter)
            .into_iter()
            .cloned()
            .collect()
    }

    pub fn clear(&self, filter: &Filter) -> usize {
        self.change(|registry| registry.clear(filter))
    }

    pub fn delete(&self, id: AutocmdId) -> Result<(), Error> {
        self.change(|registry| registry.delete(id))
    }

    pub fn forget_buffer(&self, buffer: DocId) {
        self.change(|registry| registry.forget_buffer(buffer));
    }

    pub fn create_group(&self, name: &str, clear: bool) -> GroupId {
        self.change(|registry| registry.create_group(name, clear))
    }

    pub fn delete_group(&self, id: GroupId) -> Result<(), Error> {
        self.change(|registry| registry.delete_group(id))
    }

    pub fn delete_group_by_name(&self, name: &str) -> Result<(), Error> {
        self.change(|registry| registry.delete_group_by_name(name))
    }

    pub fn group(&self, name: &str) -> Option<GroupId> {
        self.state.borrow().registry.group(name)
    }

    pub fn group_name(&self, id: GroupId) -> Option<String> {
        self.state
            .borrow()
            .registry
            .group_name(id)
            .map(str::to_string)
    }

    pub fn groups(&self) -> Vec<(GroupId, String)> {
        self.state
            .borrow()
            .registry
            .groups()
            .map(|(id, name)| (id, name.to_string()))
            .collect()
    }

    fn change<T>(&self, run: impl FnOnce(&mut Registry<Action>) -> T) -> T {
        let keepalive: Vec<_> = self
            .state
            .borrow()
            .registry
            .rows
            .iter()
            .map(|row| row.action.clone())
            .collect();
        let result = {
            let mut state = self.state.borrow_mut();
            let result = run(&mut state.registry);
            let keys: HashSet<_> = state.registry.rows.iter().map(|row| row.key).collect();
            state.compiled.retain(|key, _| keys.contains(key));
            result
        };
        drop(keepalive);
        result
    }
}
