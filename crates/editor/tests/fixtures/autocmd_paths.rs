#[rustfmt::skip]
pub const PATHS: &[(&str, &str)] = &[
    ("./name", "$CWD/name"),
    ("././name", "$CWD/name"),
    ("dir/./name", "$CWD/dir/./name"),
    ("a//b", "$CWD/a//b"),
    ("//a/b", "//a/b"),
    ("~/name", "~/name"),
    ("~abc/name", "~abc/name"),
    ("http://a/b", "http://a/b"),
    ("term://a/b", "term://a/b"),
    ("file:///a/b", "file:///a/b"),
    ("../name", "$PARENT/name"),
    ("./../name", "$PARENT/name"),
    (".", "$CWD"),
    ("target/../child", "$CWD/child"),
    ("target/./child", "$CWD/target/child"),
    ("target//child", "$CWD/target/child"),
];
