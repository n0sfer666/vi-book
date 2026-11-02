#[path = "autocmd/common.rs"]
mod common;

use common::{add, user};
use v_ide_editor::autocmd::{Attributes, EventName, Filter, GroupFilter, Registry, Selector};

#[test]
fn events_accept_case_insensitive_names_and_canonicalize_native_aliases() {
    for (input, expected) in [
        ("user", "User"),
        ("BUFREAD", "BufReadPost"),
        ("BufWrite", "BufWritePre"),
        ("BufCreate", "BufAdd"),
        ("FileEncoding", "EncodingChanged"),
    ] {
        assert_eq!(EventName::parse(input).unwrap().as_str(), expected);
    }
    assert!(EventName::parse("BadEvent").is_err());
    assert!(EventName::parse("User ").is_err());
}

#[test]
fn groups_keep_identity_when_cleared_and_get_a_new_identity_after_deletion() {
    let mut registry = Registry::new();
    let group = registry.create_group("Group", false);
    let attrs = Attributes {
        group: Some(group),
        ..Attributes::default()
    };
    let id = add(
        &mut registry,
        user(),
        Selector::File("*".into()),
        attrs,
        "one",
    );
    assert_eq!(registry.create_group("Group", false), group);
    assert_eq!(registry.entries(&Filter::default()).len(), 1);
    assert_eq!(registry.create_group("Group", true), group);
    assert!(registry.entries(&Filter::default()).is_empty());
    assert!(registry.delete(id).is_err());
    registry.delete_group(group).unwrap();
    assert_ne!(registry.create_group("Group", false), group);
    assert!(
        registry
            .define(
                Attributes {
                    group: Some(group),
                    ..Attributes::default()
                },
                "bad"
            )
            .is_err()
    );
    for name in ["", "END", "space group"] {
        let id = registry.create_group(name, false);
        assert_eq!(registry.group(name), Some(id));
    }
}

#[test]
fn filter_constraints_intersect_and_pattern_filters_compare_definitions_literally() {
    let mut registry = Registry::new();
    let group = registry.create_group("G", false);
    add(
        &mut registry,
        user(),
        Selector::File("*.rs".into()),
        Attributes {
            group: Some(group),
            nested: true,
            description: Some("grouped".into()),
            once: false,
        },
        "group",
    );
    add(
        &mut registry,
        user(),
        Selector::File("*.rs".into()),
        Attributes::default(),
        "default",
    );
    add(
        &mut registry,
        EventName::parse("BufEnter").unwrap(),
        Selector::File("*.txt".into()),
        Attributes::default(),
        "other",
    );
    let filter = Filter {
        group: GroupFilter::Group(group),
        selectors: Some(vec![Selector::File("*.rs".into())]),
        ..Filter::default()
    };
    assert_eq!(
        registry.entries(&filter)[0]
            .attributes
            .description
            .as_deref(),
        Some("grouped")
    );
    assert!(registry.entries(&filter)[0].attributes.nested);
    assert_eq!(registry.clear(&filter), 1);
    assert_eq!(
        registry
            .entries(&Filter {
                group: GroupFilter::Default,
                ..Filter::default()
            })
            .len(),
        2
    );
    let rows = registry.entries(&Filter::default());
    assert_eq!(rows[0].event.as_str(), "BufEnter");
    assert_eq!(rows[1].action, "default");
}

#[test]
fn definition_errors_preserve_valid_prefix_and_ex_rows_have_no_public_api_id() {
    let mut registry = Registry::new();
    let result = (|| {
        let mut definition = registry.define(Attributes::default(), "prefix")?;
        definition.push(user(), Selector::File("one".into()));
        let event = EventName::parse("BadEvent")?;
        definition.push(event, Selector::File("two".into()));
        Ok::<_, v_ide_editor::autocmd::Error>(())
    })();
    assert!(result.is_err());
    let mut ex = registry.define_ex(Attributes::default(), "ex").unwrap();
    assert_eq!(ex.id(), None);
    ex.push(user(), Selector::File("*".into()));
    drop(ex);
    let rows = registry.entries(&Filter::default());
    assert_eq!(rows.len(), 2);
    assert!(rows[0].id.is_some());
    assert_eq!(rows[1].id, None);
}

#[test]
fn listings_use_native_case_insensitive_event_order_even_with_reversed_filters() {
    let mut registry = Registry::new();
    let names = ["User", "CmdUndefined", "CmdlineEnter", "BufEnter"];
    for name in names {
        add(
            &mut registry,
            EventName::parse(name).unwrap(),
            Selector::File("*".into()),
            Attributes::default(),
            name,
        );
    }
    for events in [
        None,
        Some(
            names
                .into_iter()
                .map(|name| EventName::parse(name).unwrap())
                .collect(),
        ),
    ] {
        let rows = registry.entries(&Filter {
            events,
            ..Filter::default()
        });
        let order: Vec<_> = rows.iter().map(|row| row.event.as_str()).collect();
        assert_eq!(order, ["BufEnter", "CmdlineEnter", "CmdUndefined", "User"]);
    }
}
