use super::{Error, Filter, GroupFilter, GroupId, Registry};

impl<A> Registry<A> {
    pub fn group(&self, name: &str) -> Option<GroupId> {
        self.groups
            .iter()
            .find(|(_, candidate)| candidate == name)
            .map(|(id, _)| *id)
    }

    pub fn group_name(&self, id: GroupId) -> Option<&str> {
        self.groups
            .iter()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, name)| name.as_str())
    }

    pub fn groups(&self) -> impl Iterator<Item = (GroupId, &str)> {
        self.groups.iter().map(|(id, name)| (*id, name.as_str()))
    }

    pub fn create_group(&mut self, name: &str, clear: bool) -> GroupId {
        if let Some(id) = self.group(name) {
            if clear {
                self.clear(&Filter {
                    group: GroupFilter::Group(id),
                    ..Filter::default()
                });
            }
            return id;
        }
        let id = GroupId(self.next_group);
        self.next_group += 1;
        self.groups.push((id, name.to_string()));
        id
    }

    pub fn delete_group(&mut self, id: GroupId) -> Result<(), Error> {
        if self.group_name(id).is_none() {
            return Err(Error::GroupId(id));
        }
        self.clear(&Filter {
            group: GroupFilter::Group(id),
            ..Filter::default()
        });
        self.groups.retain(|(candidate, _)| *candidate != id);
        Ok(())
    }

    pub fn delete_group_by_name(&mut self, name: &str) -> Result<(), Error> {
        let id = self
            .group(name)
            .ok_or_else(|| Error::GroupName(name.to_string()))?;
        self.delete_group(id)
    }
}
