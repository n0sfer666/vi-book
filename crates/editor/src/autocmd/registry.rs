use super::{
    AutocmdId, Error, EventName, Filter, GroupFilter, GroupId, Record, RecordId, Selector,
};
use crate::workspace::DocId;

pub struct Registry<A> {
    pub(super) rows: Vec<Record<A>>,
    pub(super) groups: Vec<(GroupId, String)>,
    pub(super) next_group: u64,
    pub(super) next_id: u64,
    pub(super) next_record: u64,
}

impl<A> Default for Registry<A> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            groups: Vec::new(),
            next_group: 1,
            next_id: 1,
            next_record: 1,
        }
    }
}

impl<A> Registry<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self, filter: &Filter) -> Vec<&Record<A>> {
        let mut rows: Vec<_> = self.rows.iter().filter(|row| filter.matches(row)).collect();
        rows.sort_by_key(|row| row.event);
        rows
    }

    pub fn record(&self, key: RecordId) -> Option<&Record<A>> {
        self.rows.iter().find(|row| row.key == key)
    }

    pub fn candidates(&self, event: EventName, buffer: DocId, group: GroupFilter) -> Vec<RecordId> {
        self.rows
            .iter()
            .filter(|row| {
                row.event == event
                    && group.matches(row.attributes.group)
                    && match &row.selector {
                        Selector::File(_) => true,
                        Selector::Buffer(id) => *id == buffer,
                    }
            })
            .map(|row| row.key)
            .collect()
    }

    pub fn clear(&mut self, filter: &Filter) -> usize {
        let before = self.rows.len();
        self.rows.retain(|row| !filter.matches(row));
        before - self.rows.len()
    }

    pub fn delete(&mut self, id: AutocmdId) -> Result<(), Error> {
        if self.clear(&Filter {
            id: Some(id),
            ..Filter::default()
        }) == 0
        {
            return Err(Error::AutocmdId(id));
        }
        Ok(())
    }

    pub fn remove_record(&mut self, key: RecordId) -> bool {
        let before = self.rows.len();
        self.rows.retain(|row| row.key != key);
        before != self.rows.len()
    }

    pub fn forget_buffer(&mut self, buffer: DocId) {
        self.rows
            .retain(|row| row.selector != Selector::Buffer(buffer));
    }
}

impl<A: Clone> Registry<A> {
    pub fn begin(&mut self, key: RecordId) -> Option<Record<A>> {
        let index = self.rows.iter().position(|row| row.key == key)?;
        let row = self.rows[index].clone();
        if row.attributes.once {
            self.rows.remove(index);
        }
        Some(row)
    }
}
