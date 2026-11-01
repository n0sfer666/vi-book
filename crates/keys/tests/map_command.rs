use v_ide_keys::{MapCommand, MapMode, MapRhs, parse_keys};

#[test]
fn complete_command_family_selects_modes_and_remapping() {
    let groups = [
        (
            "",
            vec![MapMode::Normal, MapMode::Visual, MapMode::OpPending],
        ),
        ("n", vec![MapMode::Normal]),
        ("v", vec![MapMode::Visual]),
        ("x", vec![MapMode::Visual]),
        ("o", vec![MapMode::OpPending]),
        ("i", vec![MapMode::Insert]),
        ("c", vec![MapMode::Cmdline]),
    ];
    for (prefix, modes) in groups {
        for (suffix, remap) in [("map", true), ("noremap", false)] {
            let input = format!("{prefix}{suffix} Q dd");
            let parsed = MapCommand::parse(&input, ' ').unwrap().unwrap();
            assert_eq!(
                parsed,
                MapCommand::Map {
                    modes: modes.clone(),
                    lhs: parse_keys("Q"),
                    rhs: MapRhs::Keys(parse_keys("dd")),
                    remap,
                    nowait: false,
                    unique: false,
                },
                "{input}"
            );
        }
        assert_eq!(
            MapCommand::parse(&format!("{prefix}unmap Q"), ' ').unwrap(),
            Some(MapCommand::Unmap {
                modes: modes.clone(),
                lhs: parse_keys("Q")
            })
        );
        assert_eq!(
            MapCommand::parse(&format!("{prefix}mapclear"), ' ').unwrap(),
            Some(MapCommand::Clear { modes })
        );
    }
}

#[test]
fn bang_family_targets_insert_and_cmdline() {
    for command in ["map! Q dd", "noremap! Q dd", "unmap! Q", "mapclear!"] {
        let parsed = MapCommand::parse(command, ' ').unwrap().unwrap();
        let modes = match parsed {
            MapCommand::Map { modes, .. }
            | MapCommand::Unmap { modes, .. }
            | MapCommand::Clear { modes } => modes,
            other => panic!("{other:?}"),
        };
        assert_eq!(modes, vec![MapMode::Insert, MapMode::Cmdline]);
    }
}

#[test]
fn only_full_supported_names_are_recognized() {
    for input in [
        "nn Q dd",
        "nno Q dd",
        "smap Q dd",
        "lmap Q dd",
        "tmap Q dd",
        "help",
        "nmap! Q dd",
    ] {
        assert_eq!(MapCommand::parse(input, ' ').unwrap(), None, "{input}");
    }
}

#[test]
fn flags_and_leader_are_parsed_without_changing_command_text() {
    assert_eq!(
        MapCommand::parse(
            "nmap <silent> <special> <nowait> <unique> <Leader>x <cMd>new <Leader> note<cr>",
            ','
        )
        .unwrap(),
        Some(MapCommand::Map {
            modes: vec![MapMode::Normal],
            lhs: parse_keys(",x"),
            rhs: MapRhs::Cmd(String::from("new <Leader> note")),
            remap: true,
            nowait: true,
            unique: true
        })
    );
    assert_eq!(
        MapCommand::parse("noremap <leader>x <Leader>y", ',').unwrap(),
        Some(MapCommand::Map {
            modes: vec![MapMode::Normal, MapMode::Visual, MapMode::OpPending],
            lhs: parse_keys(",x"),
            rhs: MapRhs::Keys(parse_keys(",y")),
            remap: false,
            nowait: false,
            unique: false
        })
    );
}

#[test]
fn rhs_keeps_spaces_pipes_and_colon_keys() {
    for rhs in [":w<CR>", "a b  ", "a|b", "<Space><lt>"] {
        let parsed = MapCommand::parse(&format!("nmap\tQ\t{rhs}"), ' ')
            .unwrap()
            .unwrap();
        assert!(
            matches!(parsed, MapCommand::Map { rhs: MapRhs::Keys(keys), .. } if keys == parse_keys(rhs))
        );
    }
    assert_eq!(MapRhs::parse("<nOp>", ' ').unwrap(), MapRhs::Keys(vec![]));
}

#[test]
fn listing_is_distinct_from_mutation() {
    assert_eq!(
        MapCommand::parse("map", ' ').unwrap(),
        Some(MapCommand::List {
            modes: vec![MapMode::Normal, MapMode::Visual, MapMode::OpPending],
            lhs: vec![],
        })
    );
    assert_eq!(
        MapCommand::parse("nmap x", ' ').unwrap(),
        Some(MapCommand::List {
            modes: vec![MapMode::Normal],
            lhs: parse_keys("x"),
        })
    );
}

#[test]
fn malformed_arguments_report_vim_errors() {
    for input in ["unmap", "unmap a b", "mapclear extra"] {
        assert_eq!(
            MapCommand::parse(input, ' ').unwrap_err().to_string(),
            "E474: Invalid argument",
            "{input}"
        );
    }
    for flag in ["buffer", "expr", "script"] {
        let input = format!("nmap <{flag}> Q dd");
        assert_eq!(
            MapCommand::parse(&input, ' ').unwrap_err().to_string(),
            format!("E474: Invalid argument (<{flag}> not supported)")
        );
    }
    assert_eq!(
        MapRhs::parse("<Cmd>w", ' ').unwrap_err().to_string(),
        "E1255: <Cmd> mapping must end with <CR>"
    );
}
