#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Autocmd,
    Augroup,
    Doautocmd,
    Set,
    Setlocal,
    Setglobal,
    Enew,
    Buffer,
    File,
    Edit,
    Write,
    Unknown,
}

pub(super) struct Command<'a> {
    pub kind: Kind,
    pub bang: bool,
    pub args: &'a str,
    pub raw: &'a str,
}

pub(super) fn take(mut input: &str) -> Option<(Command<'_>, &str)> {
    loop {
        input = input.trim_start_matches([' ', '\t', '\n', '\r', ':', '|']);
        if input.is_empty() {
            return None;
        }
        if !input.starts_with('"') {
            break;
        }
        input = &input[input.find('\n')? + 1..];
    }
    let end = input
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(input.len());
    let name = &input[..end];
    let kind = kind(name);
    let bang = input[end..].starts_with('!');
    let start = end + usize::from(bang);
    let mut escape = false;
    let mut stop = input.len();
    let mut next = input.len();
    for (index, ch) in input[start..].char_indices() {
        let index = index + start;
        if ch == '\n' {
            stop = index;
            next = index + 1;
            break;
        }
        if kind == Kind::Autocmd {
            continue;
        }
        if escape {
            escape = false;
            continue;
        }
        if matches!(ch, '\\' | '\u{16}') {
            escape = true;
            continue;
        }
        if ch == '|' {
            stop = index;
            next = index + 1;
            break;
        }
        if ch == '"' {
            stop = index;
            next = input[index..]
                .find('\n')
                .map_or(input.len(), |offset| index + offset + 1);
            break;
        }
    }
    let args = if kind == Kind::Autocmd {
        input[start..stop].trim_start_matches([' ', '\t'])
    } else {
        input[start..stop].trim_matches([' ', '\t', '\r'])
    };
    Some((
        Command {
            kind,
            bang,
            args,
            raw: &input[..stop],
        },
        &input[next..],
    ))
}

fn kind(name: &str) -> Kind {
    for (canonical, minimum, kind) in [
        ("autocmd", 2, Kind::Autocmd),
        ("augroup", 3, Kind::Augroup),
        ("doautocmd", 2, Kind::Doautocmd),
        ("set", 2, Kind::Set),
        ("setlocal", 4, Kind::Setlocal),
        ("setglobal", 4, Kind::Setglobal),
        ("enew", 3, Kind::Enew),
        ("buffer", 1, Kind::Buffer),
        ("file", 1, Kind::File),
        ("edit", 1, Kind::Edit),
        ("write", 1, Kind::Write),
    ] {
        if name.len() >= minimum && canonical.starts_with(name) {
            return kind;
        }
    }
    Kind::Unknown
}

pub(super) fn word(input: &str) -> (&str, &str) {
    let input = input.trim_start_matches([' ', '\t']);
    let end = input.find([' ', '\t']).unwrap_or(input.len());
    (&input[..end], input[end..].trim_start_matches([' ', '\t']))
}
