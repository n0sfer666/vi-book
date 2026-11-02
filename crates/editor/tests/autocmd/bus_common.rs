use v_ide_editor::{
    autocmd::{Action, Attributes, AutocmdId, Bus, EventName, Names, Occurrence, Selector},
    workspace::DocId,
};

pub fn user() -> EventName {
    EventName::parse("User").unwrap()
}

pub fn event(buffer: DocId, name: &str) -> Occurrence {
    Occurrence::new(user(), Some(buffer), Names::same(name))
}

pub fn add(bus: &Bus, pattern: &str, action: Action) -> AutocmdId {
    bus.define(
        Attributes::default(),
        action,
        &["User"],
        &[Selector::File(pattern.into())],
    )
    .unwrap()
}
