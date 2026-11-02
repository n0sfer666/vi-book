use super::{Change, Error, ErrorKind, Name, Operation, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument {
    List { all: bool },
    ResetAll,
    Change(Change),
}

pub(crate) struct Arguments<'a> {
    rest: &'a str,
    empty: bool,
    stopped: bool,
}

impl<'a> Arguments<'a> {
    pub(crate) fn new(rest: &'a str) -> Self {
        Self {
            rest,
            empty: rest.trim_matches([' ', '\t']).is_empty(),
            stopped: false,
        }
    }
}

impl Iterator for Arguments<'_> {
    type Item = Result<Argument, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stopped {
            return None;
        }
        if self.empty {
            self.stopped = true;
            return Some(Ok(Argument::List { all: false }));
        }
        let start = self.rest.trim_start_matches([' ', '\t']);
        if start.is_empty() {
            return None;
        }
        let end = start.find([' ', '\t']).unwrap_or(start.len());
        let first = &start[..end];
        let tail = start[end..].trim_start_matches([' ', '\t']);
        let operator = tail.starts_with(['=', ':', '?', '!', '&', '<'])
            || ["+=", "-=", "^="]
                .iter()
                .any(|prefix| tail.starts_with(prefix));
        let (token, end) = if operator && first.chars().all(|c| c.is_ascii_alphabetic()) {
            let length = tail.find([' ', '\t']).unwrap_or(tail.len());
            (
                format!("{first}{}", &tail[..length]),
                start.len() - tail.len() + length,
            )
        } else {
            (first.to_string(), end)
        };
        let error_end = if end < start.len() {
            end + start[end..].chars().next().map_or(0, char::len_utf8)
        } else {
            end
        };
        let raw = &start[..error_end];
        self.rest = &start[end..];
        let result = parse(&token, raw);
        self.stopped = result.is_err();
        Some(result)
    }
}

fn parse(token: &str, raw: &str) -> Result<Argument, Error> {
    if token == "all" {
        return Ok(Argument::List { all: true });
    }
    if matches!(token, "all&" | "all&vi" | "all&vim") {
        return Ok(Argument::ResetAll);
    }
    let end = token
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(token.len());
    let word = &token[..end];
    let suffix = &token[end..];
    let (word, prefix) = if let Some(word) = word.strip_prefix("no") {
        (word, Some(false))
    } else if let Some(word) = word.strip_prefix("inv") {
        (word, Some(true))
    } else {
        (word, None)
    };
    let name = Name::parse(word).ok_or_else(|| Error::new(ErrorKind::Unknown, raw))?;
    let boolean = matches!(name.default_value(), Value::Bool(_));
    if prefix.is_some() && !boolean {
        return Err(Error::new(ErrorKind::Invalid, raw));
    }
    let operation = match suffix {
        "?" => Operation::Query,
        "&" | "&vi" | "&vim" => Operation::Reset,
        "<" => Operation::CopyGlobal,
        "!" if boolean => Operation::Toggle,
        "" if !boolean => Operation::Query,
        "" => match prefix {
            Some(false) => Operation::Set(Value::Bool(false)),
            Some(true) => Operation::Toggle,
            None => Operation::Set(Value::Bool(true)),
        },
        _ if boolean
            && (suffix.starts_with(['=', ':'])
                || ["+=", "-=", "^="]
                    .iter()
                    .any(|operator| suffix.starts_with(operator))) =>
        {
            return Err(Error::new(ErrorKind::Invalid, raw));
        }
        _ if !boolean => numeric(suffix, raw)?,
        _ => return Err(Error::new(ErrorKind::Trailing, raw)),
    };
    Ok(Argument::Change(Change {
        name,
        operation,
        text: raw.to_string(),
    }))
}

fn numeric(suffix: &str, raw: &str) -> Result<Operation, Error> {
    let (operator, text) = if let Some(text) = suffix.strip_prefix("+=") {
        ("+", text)
    } else if let Some(text) = suffix.strip_prefix("-=") {
        ("-", text)
    } else if let Some(text) = suffix.strip_prefix("^=") {
        ("^", text)
    } else if let Some(text) = suffix.strip_prefix(['=', ':']) {
        ("=", text)
    } else {
        return Err(Error::new(ErrorKind::Trailing, raw));
    };
    let negative = text.starts_with('-');
    let digits = text.strip_prefix('-').unwrap_or(text);
    let (digits, radix) = if let Some(digits) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        (digits, 16)
    } else if let Some(digits) = digits
        .strip_prefix("0o")
        .or_else(|| digits.strip_prefix("0O"))
    {
        (digits, 8)
    } else if let Some(digits) = digits
        .strip_prefix("0b")
        .or_else(|| digits.strip_prefix("0B"))
    {
        (digits, 2)
    } else if digits.starts_with('0')
        && digits.len() > 1
        && digits.chars().all(|c| ('0'..='7').contains(&c))
    {
        (digits, 8)
    } else {
        (digits, 10)
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return Err(Error::new(ErrorKind::Number, raw));
    }
    let number =
        i64::from_str_radix(digits, radix).map_err(|_| Error::new(ErrorKind::Invalid, raw))?;
    let number = if negative { -number } else { number };
    Ok(match operator {
        "+" => Operation::Add(number),
        "-" => Operation::Subtract(number),
        "^" => Operation::Multiply(number),
        _ => Operation::Set(Value::Number(number)),
    })
}
