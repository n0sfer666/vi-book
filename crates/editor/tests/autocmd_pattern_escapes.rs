use v_ide_editor::autocmd::Pattern;

#[test]
fn escaped_letters_without_vim_regexp_meaning_remain_literal() {
    for ch in "BEGJNOQRTYgjqy".chars() {
        let source = format!("x\\{ch}z");
        let name = format!("x{ch}z");
        let compiled = Pattern::compile(&source).unwrap();
        assert!(compiled.matches(&name, &name, false), "{source:?}");
        assert!(!compiled.matches("x!z", "x!z", false), "{source:?}");
    }
}
