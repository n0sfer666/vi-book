use std::time::Instant;

use v_ide_keys::{Key, Keymap, MapMode, Mapping, Rhs, Step, TIMEOUT, Typeahead, parse_keys};

fn insert(map: &mut Keymap<&'static str>, lhs: &str, rhs: Rhs<&'static str>) {
    map.insert(
        MapMode::Normal,
        &parse_keys(lhs),
        Mapping {
            rhs,
            remap: false,
            nowait: false,
        },
    )
    .unwrap();
}

#[test]
fn builtin_waits_behind_a_user_mapping_then_uses_the_same_deadline() {
    let now = Instant::now();
    let mut map = Keymap::default();
    insert(&mut map, "ax", Rhs::Action("mapping"));
    let mut queue = Typeahead::new(TIMEOUT);
    let leader = parse_keys("ab");
    queue.feed(Key::char('a'));
    assert_eq!(
        queue.next_with_builtin(
            &map,
            Some(MapMode::Normal),
            false,
            now,
            Some((&leader, "menu"))
        ),
        Step::Wait
    );
    assert_eq!(
        queue.next_with_builtin(
            &map,
            Some(MapMode::Normal),
            false,
            now + TIMEOUT,
            Some((&leader, "menu"))
        ),
        Step::Key(Key::char('a'))
    );
    assert!(queue.deadline().is_none());
}

#[test]
fn user_prefix_mapping_takes_precedence_over_a_longer_builtin() {
    let now = Instant::now();
    let mut map = Keymap::default();
    insert(&mut map, "a", Rhs::Action("mapping"));
    let mut queue = Typeahead::new(TIMEOUT);
    let leader = parse_keys("ab");
    for key in parse_keys("ab") {
        queue.feed(key);
    }
    assert_eq!(
        queue.next_with_builtin(
            &map,
            Some(MapMode::Normal),
            false,
            now,
            Some((&leader, "menu"))
        ),
        Step::Action("mapping")
    );
    assert_eq!(
        queue.next_with_builtin(
            &map,
            Some(MapMode::Normal),
            false,
            now,
            Some((&leader, "menu"))
        ),
        Step::Key(Key::char('b'))
    );
}

#[test]
fn noremap_output_can_invoke_a_builtin_but_not_cross_a_command_boundary() {
    let now = Instant::now();
    let mut map = Keymap::default();
    insert(&mut map, "Q", Rhs::Keys(parse_keys("ab")));
    insert(
        &mut map,
        "Z",
        Rhs::Sequence(vec![
            Rhs::Keys(parse_keys("a")),
            Rhs::Action("command"),
            Rhs::Keys(parse_keys("b")),
        ]),
    );
    let leader = parse_keys("ab");
    let mut queue = Typeahead::new(TIMEOUT);
    queue.feed(Key::char('Q'));
    assert_eq!(
        queue.next_with_builtin(
            &map,
            Some(MapMode::Normal),
            false,
            now,
            Some((&leader, "menu"))
        ),
        Step::Action("menu")
    );
    queue.feed(Key::char('Z'));
    for expected in [
        Step::Key(Key::char('a')),
        Step::Action("command"),
        Step::Key(Key::char('b')),
    ] {
        assert_eq!(
            queue.next_with_builtin(
                &map,
                Some(MapMode::Normal),
                false,
                now,
                Some((&leader, "menu"))
            ),
            expected
        );
    }
}
