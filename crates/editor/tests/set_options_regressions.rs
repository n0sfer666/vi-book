use v_ide_editor::options::{Argument, SetCommand, Value};

fn error(text: &str) -> String {
    match SetCommand::parse(&format!("set {text}"))
        .unwrap()
        .arguments()
        .next()
        .unwrap()
    {
        Err(error) => error.to_string(),
        Ok(Argument::Change(change)) => change
            .value(Value::Number(1000), Value::Number(1000))
            .unwrap_err()
            .to_string(),
        _ => panic!("expected error"),
    }
}

#[test]
fn unicode_whitespace_is_not_an_ex_option_separator() {
    for separator in ['\u{a0}', '\u{2003}'] {
        assert!(error(&format!("nu{separator}ro")).starts_with("E488"));
        assert!(SetCommand::parse(&format!("set{separator}nu")).is_none());
    }
    assert_eq!(
        SetCommand::parse("set nu\tro").unwrap().arguments().count(),
        2
    );
}

#[test]
fn incomplete_boolean_arithmetic_is_a_trailing_character_error() {
    for argument in ["nu+", "nu-", "nu^", "nu+x"] {
        assert!(error(argument).starts_with("E488"), "{argument}");
    }
}

#[test]
fn signed_integer_overflow_is_distinct_from_a_negative_timeout() {
    assert!(error("tm=-2147483649").starts_with("E474"));
    assert!(error("tm=-2147483648").starts_with("E487"));
}
