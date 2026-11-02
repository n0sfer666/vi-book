use super::{Attributes, AutocmdId, Error, EventName, Record, RecordId, Registry, Selector};

pub struct Definition<'a, A> {
    registry: &'a mut Registry<A>,
    id: Option<AutocmdId>,
    attributes: Attributes,
    action: A,
}

impl<A> Registry<A> {
    pub fn define(
        &mut self,
        attributes: Attributes,
        action: A,
    ) -> Result<Definition<'_, A>, Error> {
        self.validate_group(&attributes)?;
        let id = AutocmdId(self.next_id);
        self.next_id += 1;
        Ok(Definition {
            registry: self,
            id: Some(id),
            attributes,
            action,
        })
    }

    pub fn define_ex(
        &mut self,
        attributes: Attributes,
        action: A,
    ) -> Result<Definition<'_, A>, Error> {
        self.validate_group(&attributes)?;
        Ok(Definition {
            registry: self,
            id: None,
            attributes,
            action,
        })
    }

    fn validate_group(&self, attributes: &Attributes) -> Result<(), Error> {
        if let Some(id) = attributes.group
            && self.group_name(id).is_none()
        {
            return Err(Error::GroupId(id));
        }
        Ok(())
    }
}

impl<A> Definition<'_, A> {
    pub fn id(&self) -> Option<AutocmdId> {
        self.id
    }
}

impl<A: Clone> Definition<'_, A> {
    pub fn push(&mut self, event: EventName, selector: Selector) -> RecordId {
        let key = RecordId(self.registry.next_record);
        self.registry.next_record += 1;
        self.registry.rows.push(Record {
            key,
            id: self.id,
            event,
            selector,
            attributes: self.attributes.clone(),
            action: self.action.clone(),
        });
        key
    }
}
