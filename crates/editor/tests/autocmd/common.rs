use v_ide_editor::autocmd::{Attributes, AutocmdId, EventName, Registry, Selector};

pub fn user() -> EventName {
    EventName::parse("User").unwrap()
}

pub fn add(
    registry: &mut Registry<&'static str>,
    event: EventName,
    selector: Selector,
    attributes: Attributes,
    action: &'static str,
) -> AutocmdId {
    let mut definition = registry.define(attributes, action).unwrap();
    let id = definition.id().unwrap();
    definition.push(event, selector);
    id
}
