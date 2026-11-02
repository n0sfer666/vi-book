use std::{
    cell::Cell,
    rc::{Rc, Weak},
};
use v_ide_editor::{
    autocmd::{Action, Attributes, Bus, Callback, Filter, Invocation, Selector},
    document::Document,
    execution::Context,
    workspace::Workspace,
};

struct Reenter {
    bus: Weak<Bus>,
    dropped: Rc<Cell<usize>>,
}
impl Callback for Reenter {
    fn run(&self, _: &Context<'_>, _: &Invocation<'_>) -> Result<bool, String> {
        Ok(false)
    }
}
impl Drop for Reenter {
    fn drop(&mut self) {
        if let Some(bus) = self.bus.upgrade() {
            drop(bus.entries(&Filter::default()));
        }
        self.dropped.set(self.dropped.get() + 1);
    }
}

#[test]
fn callback_destructors_can_reenter_after_clear_delete_group_replace_and_forget() {
    let workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    for operation in 0..6 {
        let bus = Rc::new(Bus::new(false));
        let dropped = Rc::new(Cell::new(0));
        let group = bus.create_group("group", false);
        let action = Action::callback(Reenter {
            bus: Rc::downgrade(&bus),
            dropped: dropped.clone(),
        });
        let id = bus
            .define(
                Attributes {
                    group: Some(group),
                    ..Attributes::default()
                },
                action,
                &["User"],
                &[Selector::Buffer(buffer)],
            )
            .unwrap();
        match operation {
            0 => {
                bus.clear(&Filter::default());
            }
            1 => bus.delete(id).unwrap(),
            2 => bus.delete_group(group).unwrap(),
            3 => bus.delete_group_by_name("group").unwrap(),
            4 => {
                bus.create_group("group", true);
            }
            _ => bus.forget_buffer(buffer),
        }
        assert_eq!(dropped.get(), 1, "{operation}");
    }
}

#[test]
fn failed_registration_drops_callbacks_after_releasing_the_registry_borrow() {
    let bus = Rc::new(Bus::new(false));
    let dropped = Rc::new(Cell::new(0));
    let action = Action::callback(Reenter {
        bus: Rc::downgrade(&bus),
        dropped: dropped.clone(),
    });
    assert!(
        bus.define(
            Attributes::default(),
            action,
            &["User"],
            &[Selector::File("{".into())]
        )
        .is_err()
    );
    assert_eq!(dropped.get(), 1);
    assert!(bus.entries(&Filter::default()).is_empty());
}
