use v_ide_editor::options::{Access, Argument, Name, Scope, SetCommand, Value};

fn change(text: &str, current: Value, global: Value) -> Result<Option<Value>, String> {
    let command = SetCommand::parse(&format!("set {text}")).unwrap();
    let argument = command
        .arguments()
        .next()
        .unwrap()
        .map_err(|e| e.to_string())?;
    match argument {
        Argument::Change(change) => change.value(current, global).map_err(|e| e.to_string()),
        _ => panic!("expected option change"),
    }
}

#[test]
fn registry_declares_names_aliases_scopes_and_defaults() {
    assert_eq!(Name::parse("tm"), Some(Name::Timeoutlen));
    assert_eq!(Name::parse("ro"), Some(Name::Readonly));
    assert_eq!(Name::parse("nu"), Some(Name::Number));
    assert_eq!(Name::parse("not-an-option"), None);
    assert_eq!(Name::Timeoutlen.scope(), Scope::Global);
    assert_eq!(Name::Readonly.scope(), Scope::Buffer);
    assert_eq!(Name::Number.scope(), Scope::Window);
    assert_eq!(Name::Timeoutlen.default_value(), Value::Number(1000));
    assert_eq!(Name::Number.default_value(), Value::Bool(false));
    assert_eq!(Name::Readonly.default_value(), Value::Bool(false));
}

#[test]
fn set_families_select_access_without_accepting_other_commands() {
    for (text, access) in [
        ("set nu", Access::Effective),
        ("setlocal nu", Access::Local),
        ("setglobal nu", Access::Global),
    ] {
        assert_eq!(SetCommand::parse(text).unwrap().access, access);
    }
    for text in ["setup", "setlocalx nu", "setglobalx", "se nu", "map Q j"] {
        assert!(SetCommand::parse(text).is_none());
    }
}

#[test]
fn boolean_forms_and_queries_match_vim() {
    for (text, current, expected) in [
        ("nu", false, Some(true)),
        ("nonu", true, Some(false)),
        ("invnumber", false, Some(true)),
        ("nu !", true, Some(false)),
        ("nu?", true, None),
        ("noro?", true, None),
        ("nu&", true, Some(false)),
        ("nu &vi", true, Some(false)),
        ("nu&vim", true, Some(false)),
        ("nu <", false, Some(true)),
    ] {
        assert_eq!(
            change(text, Value::Bool(current), Value::Bool(true)).unwrap(),
            expected.map(Value::Bool),
            "{text}"
        );
    }
}

#[test]
fn numeric_forms_include_native_radices_arithmetic_and_scope_reset() {
    for (text, expected) in [
        ("tm =37", Some(37)),
        ("tm:37", Some(37)),
        ("tm=0", Some(0)),
        ("tm=0x10", Some(16)),
        ("tm=010", Some(8)),
        ("tm=08", Some(8)),
        ("tm=0o10", Some(8)),
        ("tm=0b10", Some(2)),
        ("tm=-0", Some(0)),
        ("tm+=3", Some(1003)),
        ("tm-=3", Some(997)),
        ("tm^=3", Some(3000)),
        ("tm+=-3", Some(997)),
        ("tm?", None),
        ("tm", None),
        ("tm&vim", Some(1000)),
        ("tm<", Some(27)),
        ("tm=2147483647", Some(2147483647)),
    ] {
        assert_eq!(
            change(text, Value::Number(1000), Value::Number(27)).unwrap(),
            expected.map(Value::Number),
            "{text}"
        );
    }
}

#[test]
fn invalid_arguments_report_native_codes_and_leave_values_untouched() {
    for (text, code) in [
        ("unknown", "E518"),
        ("nu=1", "E474"),
        ("nu+=1", "E474"),
        ("tm=abc", "E521"),
        ("tm=+5", "E521"),
        ("tm=10x", "E521"),
        ("tm=-1", "E487"),
        ("tm-=2000", "E487"),
        ("tm=2147483648", "E474"),
        ("tm^=2147483647", "E474"),
        ("tm!", "E488"),
        ("notm", "E474"),
        ("invtimeoutlen", "E474"),
        ("tm = 37", "E521"),
        ("tm= 37", "E521"),
        ("tm&x", "E488"),
    ] {
        let error = change(text, Value::Number(1000), Value::Number(1000)).unwrap_err();
        assert!(error.starts_with(code), "{text}: {error}");
    }
}

#[test]
fn argument_stream_preserves_valid_prefix_and_stops_at_first_error() {
    let command = SetCommand::parse("set nu unknown noro").unwrap();
    let mut arguments = command.arguments();
    assert!(matches!(
        arguments.next().unwrap().unwrap(),
        Argument::Change(_)
    ));
    assert_eq!(
        arguments.next().unwrap().unwrap_err().to_string(),
        "E518: Unknown option: unknown "
    );
    assert!(arguments.next().is_none());
    assert_eq!(
        SetCommand::parse("set")
            .unwrap()
            .arguments()
            .next()
            .unwrap()
            .unwrap(),
        Argument::List { all: false }
    );
    assert_eq!(
        SetCommand::parse("set all")
            .unwrap()
            .arguments()
            .next()
            .unwrap()
            .unwrap(),
        Argument::List { all: true }
    );
    assert_eq!(
        SetCommand::parse("set all&")
            .unwrap()
            .arguments()
            .next()
            .unwrap()
            .unwrap(),
        Argument::ResetAll
    );
}
