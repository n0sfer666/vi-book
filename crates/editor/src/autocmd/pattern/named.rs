use super::{PatternError, unicode};

#[derive(Debug, Clone)]
pub(super) enum Named {
    Alnum,
    Alpha,
    Blank,
    Cntrl,
    Digit,
    Graph,
    Lower,
    Print,
    Punct,
    Space,
    Upper,
    Xdigit,
}

impl Named {
    pub(super) fn parse(name: &str) -> Result<Self, PatternError> {
        match name {
            "alnum" => Ok(Self::Alnum),
            "alpha" => Ok(Self::Alpha),
            "blank" => Ok(Self::Blank),
            "cntrl" => Ok(Self::Cntrl),
            "digit" => Ok(Self::Digit),
            "graph" => Ok(Self::Graph),
            "lower" => Ok(Self::Lower),
            "print" => Ok(Self::Print),
            "punct" => Ok(Self::Punct),
            "space" => Ok(Self::Space),
            "upper" => Ok(Self::Upper),
            "xdigit" => Ok(Self::Xdigit),
            _ => Err(PatternError::UnsupportedRegex(format!("[[:{name}:]]"))),
        }
    }

    pub(super) fn matches(&self, ch: char) -> bool {
        match self {
            Self::Alnum => ch.is_ascii_alphanumeric(),
            Self::Alpha => ch.is_ascii_alphabetic(),
            Self::Blank => matches!(ch, ' ' | '\t'),
            Self::Cntrl => ch.is_ascii_control(),
            Self::Digit => ch.is_ascii_digit(),
            Self::Graph => ch.is_ascii_graphic(),
            Self::Lower => unicode::lower(ch),
            Self::Print => unicode::printable(ch),
            Self::Punct => ch.is_ascii_punctuation(),
            Self::Space => ch == ' ' || ('\t'..='\r').contains(&ch),
            Self::Upper => unicode::upper(ch),
            Self::Xdigit => ch.is_ascii_hexdigit(),
        }
    }
}
