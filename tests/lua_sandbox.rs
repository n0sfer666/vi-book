use vi_book::lua::runtime::{Capability, Runtime};

#[test]
fn safe_stdlib_is_available() {
    let rt = Runtime::new().unwrap();
    assert_eq!(rt.eval_string("return string.upper('hi')").unwrap(), "HI");
    assert_eq!(rt.eval_string("return tostring(1 + 2)").unwrap(), "3");
    assert!(rt.eval_bool("return #({1, 2, 3}) == 3").unwrap());
    assert!(rt.eval_bool("return math.floor(2.7) == 2").unwrap());
}

#[test]
fn os_is_denied_by_default() {
    let rt = Runtime::new().unwrap();
    assert!(rt.eval_bool("return os == nil").unwrap());
}

#[test]
fn io_is_denied_by_default() {
    let rt = Runtime::new().unwrap();
    assert!(rt.eval_bool("return io == nil").unwrap());
}

#[test]
fn debug_and_package_are_denied() {
    let rt = Runtime::new().unwrap();
    assert!(rt.eval_bool("return debug == nil").unwrap());
    assert!(rt.eval_bool("return package == nil").unwrap());
    assert!(rt.eval_bool("return require == nil").unwrap());
    assert!(rt.eval_bool("return dofile == nil").unwrap());
    assert!(rt.eval_bool("return loadfile == nil").unwrap());
}

#[test]
fn granting_os_makes_it_available() {
    let rt = Runtime::new().unwrap();
    assert!(rt.eval_bool("return os == nil").unwrap());
    rt.grant(Capability::Os).unwrap();
    assert!(rt.eval_bool("return os ~= nil").unwrap());
    assert!(rt.eval_bool("return type(os.time()) == 'number'").unwrap());
}

#[test]
fn granting_io_makes_it_available() {
    let rt = Runtime::new().unwrap();
    assert!(rt.eval_bool("return io == nil").unwrap());
    rt.grant(Capability::Io).unwrap();
    assert!(rt.eval_bool("return io ~= nil").unwrap());
}

#[test]
fn syntax_error_returns_err_not_panic() {
    let rt = Runtime::new().unwrap();
    assert!(rt.exec("this is not lua !!!").is_err());
}

#[test]
fn runtime_error_returns_err_not_panic() {
    let rt = Runtime::new().unwrap();
    assert!(rt.exec("error('boom')").is_err());
    assert!(rt.exec("local x = nil; return x.field").is_err());
}

#[test]
fn runtime_survives_after_error() {
    let rt = Runtime::new().unwrap();
    assert!(rt.exec("error('boom')").is_err());
    assert_eq!(
        rt.eval_string("return 'still alive'").unwrap(),
        "still alive"
    );
}
