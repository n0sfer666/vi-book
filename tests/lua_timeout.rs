use vi_book::lua::host::PluginHost;
use vi_book::lua::runtime::Runtime;

#[test]
fn infinite_loop_in_exec_returns_err() {
    let rt = Runtime::new().unwrap();
    assert!(rt.exec("while true do end").is_err());
}

#[test]
fn runtime_survives_after_budget_abort() {
    let rt = Runtime::new().unwrap();
    assert!(rt.exec("local n = 0\nwhile true do n = n + 1 end").is_err());
    assert_eq!(rt.eval_string("return 'ok'").unwrap(), "ok");
}

#[test]
fn normal_work_stays_within_budget() {
    let rt = Runtime::new().unwrap();
    assert!(
        rt.exec("local s = 0\nfor i = 1, 100000 do s = s + i end")
            .is_ok()
    );
}

#[test]
fn infinite_loop_in_command_returns_err() {
    let host = PluginHost::new().unwrap();
    host.load("vibook.command('hang', function() while true do end end)")
        .unwrap();
    assert!(host.run_command("hang").is_err());
}

#[test]
fn infinite_loop_in_event_returns_err() {
    let host = PluginHost::new().unwrap();
    host.load("vibook.on('e', function() while true do end end)")
        .unwrap();
    assert!(host.fire("e").is_err());
}
