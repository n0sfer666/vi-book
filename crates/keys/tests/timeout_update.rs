use std::time::{Duration, Instant};

use v_ide_keys::{Key, Keymap, MapMode, Mapping, Rhs, Step, TIMEOUT, Typeahead, parse_keys};

fn map() -> Keymap<()> {
    let mut map = Keymap::default();
    map.insert(
        MapMode::Normal,
        &parse_keys("ab"),
        Mapping {
            rhs: Rhs::Keys(parse_keys("j")),
            remap: false,
            nowait: false,
        },
    )
    .unwrap();
    map
}

#[test]
fn timeout_change_preserves_the_start_of_an_existing_wait() {
    let now = Instant::now();
    let map = map();
    let mut queue = Typeahead::new(TIMEOUT);
    queue.feed(Key::char('a'));
    assert_eq!(
        queue.next(&map, Some(MapMode::Normal), false, now),
        Step::Wait
    );
    queue.set_timeout(Duration::from_millis(40));
    assert_eq!(queue.deadline(), Some(now + Duration::from_millis(40)));
    assert_eq!(
        queue.next(
            &map,
            Some(MapMode::Normal),
            false,
            now + Duration::from_millis(40)
        ),
        Step::Key(Key::char('a'))
    );
}

#[test]
fn zero_timeout_resolves_prefix_without_waiting_for_another_poll() {
    let now = Instant::now();
    let map = map();
    let mut queue = Typeahead::new(TIMEOUT);
    queue.set_timeout(Duration::ZERO);
    queue.feed(Key::char('a'));
    assert_eq!(
        queue.next(&map, Some(MapMode::Normal), false, now),
        Step::Key(Key::char('a'))
    );
    assert!(queue.deadline().is_none());
    queue.feed(Key::char('x'));
    assert_eq!(
        queue.next_with_builtin(
            &map,
            Some(MapMode::Normal),
            false,
            now,
            Some((&parse_keys("xy"), ()))
        ),
        Step::Key(Key::char('x'))
    );
}

#[test]
fn applying_the_same_timeout_does_not_postpone_a_wait() {
    let now = Instant::now();
    let map = map();
    let mut queue = Typeahead::new(TIMEOUT);
    queue.feed(Key::char('a'));
    assert_eq!(
        queue.next(&map, Some(MapMode::Normal), false, now),
        Step::Wait
    );
    queue.set_timeout(TIMEOUT);
    assert_eq!(queue.deadline(), Some(now + TIMEOUT));
    assert_eq!(
        queue.next(&map, Some(MapMode::Normal), false, now + TIMEOUT),
        Step::Key(Key::char('a'))
    );
}
