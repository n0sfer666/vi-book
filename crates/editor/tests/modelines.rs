use std::cell::Cell;

use v_ide_editor::{
    document::Document,
    execution::Context,
    options::{Access, Name, Value},
    workspace::Workspace,
};

#[path = "fixtures/modelines.rs"]
mod native;

#[test]
fn supported_modeline_options_match_the_native_matrix() {
    for &(label, lines, number, readonly, modeline, timeoutlen, modelines, error, count) in
        native::CASES
    {
        let mut workspace = Workspace::new(Document::new(&format!("{}\n", lines.join("\n"))));
        let seen = Cell::new(0);
        let observer = |_: &Context<'_>, _: &v_ide_editor::execution::OptionSet| {
            seen.set(seen.get() + 1);
            Ok(())
        };
        let cx = Context::with_observer(&mut workspace, &observer);
        cx.execute("set modeline modelines=5").unwrap();
        seen.set(0);
        cx.process_modelines();
        for (name, expected) in [
            (Name::Number, Value::Bool(number)),
            (Name::Readonly, Value::Bool(readonly)),
            (Name::Modeline, Value::Bool(modeline)),
            (Name::Timeoutlen, Value::Number(timeoutlen)),
            (Name::Modelines, Value::Number(modelines)),
        ] {
            assert_eq!(
                cx.option(name, Access::Effective),
                expected,
                "{label}: {name:?}"
            );
        }
        assert_eq!(
            !cx.take_messages()
                .iter()
                .all(|message| !message.contains("E5") && !message.contains("E4")),
            error,
            "{label}: diagnostics"
        );
        assert_eq!(seen.get(), count, "{label}: OptionSet count");
    }
}

#[test]
fn modeline_settings_are_local_and_scan_count_is_frozen_without_duplicate_lines() {
    let mut workspace = Workspace::new(Document::new(
        "vim: set number readonly:\n\n\nvim: set timeoutlen=17:\n",
    ));
    let count = Cell::new(0);
    let observer = |_: &Context<'_>, _: &v_ide_editor::execution::OptionSet| {
        count.set(count.get() + 1);
        Ok(())
    };
    let cx = Context::with_observer(&mut workspace, &observer);
    cx.execute("set modeline modelines=3").unwrap();
    count.set(0);
    cx.process_modelines();
    assert_eq!(count.get(), 3);
    assert_eq!(cx.option(Name::Number, Access::Global), Value::Bool(false));
    assert_eq!(
        cx.option(Name::Readonly, Access::Global),
        Value::Bool(false)
    );
    assert_eq!(
        cx.option(Name::Timeoutlen, Access::Global),
        Value::Number(17)
    );
    cx.execute("setlocal nonumber nomodeline").unwrap();
    cx.process_modelines();
    assert_eq!(cx.option(Name::Number, Access::Local), Value::Bool(false));
}

#[test]
fn option_observers_cannot_reenter_modeline_processing_and_double_apply_a_toggle() {
    let mut workspace = Workspace::new(Document::new("vim: set invnumber:"));
    let observer = |cx: &Context<'_>, event: &v_ide_editor::execution::OptionSet| {
        if event.name == Name::Number {
            cx.process_modelines();
        }
        Ok(())
    };
    let cx = Context::with_observer(&mut workspace, &observer);
    cx.execute("set modeline").unwrap();
    cx.process_modelines();
    assert_eq!(cx.option(Name::Number, Access::Local), Value::Bool(true));
}
