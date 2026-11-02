#[path = "oracle/app_snapshot.rs"]
mod app_snapshot;
#[path = "oracle/fixture.rs"]
mod fixture;
#[path = "oracle/snapshot.rs"]
mod snapshot;

fn check(topic: &str) {
    let fixture = fixture::load(topic);
    let mut failures = Vec::new();
    for case in &fixture.cases {
        let got = if matches!(
            topic,
            "map_commands" | "pager" | "lua_mappings" | "shared_windows" | "options"
        ) {
            app_snapshot::run(case)
        } else {
            snapshot::run(case)
        };
        let same = got == case.expect;
        match (&case.divergence, same) {
            (None, false) => failures.push(format!(
                "{}: keys {:?}\n  got  {:?}\n  nvim {:?}",
                case.name, case.keys, got, case.expect
            )),
            (Some(reason), true) => failures.push(format!(
                "{}: marked divergence ({reason}) but matches nvim now",
                case.name
            )),
            _ => {}
        }
    }
    assert!(
        failures.is_empty(),
        "{topic} vs nvim {}:\n{}",
        fixture.nvim_version,
        failures.join("\n")
    );
}

macro_rules! oracle_topics {
    ($($topic:ident),* $(,)?) => {
        const TOPICS: &[&str] = &[$(stringify!($topic)),*];
        $(
            #[test]
            fn $topic() {
                check(stringify!($topic));
            }
        )*
    };
}

oracle_topics!(
    counts,
    insert_mode,
    lua_mappings,
    map_commands,
    mappings,
    motions,
    operators,
    options,
    pager,
    registers,
    shared_windows,
    text_objects,
    undo_redo,
    visual,
);

#[test]
fn every_fixture_is_wired() {
    let mut found: Vec<String> = std::fs::read_dir(fixture::DIR)
        .expect("oracle dir")
        .filter_map(|entry| {
            let path = entry.expect("dir entry").path();
            let is_json = path.extension().is_some_and(|ext| ext == "json");
            is_json.then(|| {
                path.file_stem()
                    .expect("stem")
                    .to_string_lossy()
                    .into_owned()
            })
        })
        .collect();
    found.sort();
    let mut wired: Vec<String> = TOPICS.iter().map(|t| t.to_string()).collect();
    wired.sort();
    assert_eq!(found, wired, "tests/oracle/*.json vs oracle_topics!");
}
mod common;
