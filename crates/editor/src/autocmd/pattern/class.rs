use super::{PatternError, named::Named, scan::Scan, unicode};

#[derive(Debug, Clone)]
pub(super) struct Class {
    negated: bool,
    items: Vec<Item>,
}

#[derive(Debug, Clone)]
enum Item {
    Range(char, char),
    Named(Named),
}

impl Class {
    pub(super) fn parse(
        chars: &[char],
        index: &mut usize,
        scan: &Scan,
    ) -> Result<Option<Self>, PatternError> {
        let start = *index;
        let mut cursor = start;
        let negated = chars.get(cursor) == Some(&'^');
        cursor += usize::from(negated);
        if chars.get(cursor).is_some_and(|ch| matches!(ch, ']' | '-')) {
            cursor += 1;
        }
        let Some(end) = scan.class_end(cursor) else {
            return Ok(None);
        };
        let mut items = Vec::new();
        cursor = start + usize::from(negated);
        while cursor < end {
            if chars.get(cursor..cursor + 2) == Some(&['[', '=']) {
                return Err(PatternError::UnsupportedRegex("[[=...=]]".into()));
            }
            if let Some(named) = named(chars, &mut cursor, end, scan)? {
                items.push(Item::Named(named));
                continue;
            }
            let first = literal(chars, &mut cursor, end + 1)?;
            if chars.get(cursor) == Some(&'-') && cursor + 1 < end {
                while chars.get(cursor) == Some(&'-') {
                    cursor += 1;
                }
                if cursor >= end {
                    items.push(Item::Range(first, first));
                    items.push(Item::Range('-', '-'));
                    continue;
                }
                let mut names = Vec::new();
                while let Some(named) = named(chars, &mut cursor, end, scan)? {
                    names.push(named);
                }
                if !names.is_empty() {
                    items.push(Item::Range(first, first));
                    if cursor < end {
                        let last = literal(chars, &mut cursor, end + 1)?;
                        items.push(Item::Range('\0', last));
                    } else {
                        items.extend(names.into_iter().map(Item::Named));
                    }
                    continue;
                }
                let last = literal(chars, &mut cursor, end + 1)?;
                if first > last {
                    return Err(PatternError::ReverseRange);
                }
                items.push(Item::Range(first, last));
            } else {
                items.push(Item::Range(first, first));
            }
        }
        *index = end + 1;
        Ok(Some(Self { negated, items }))
    }

    pub(super) fn matches(&self, ch: char, ignore_case: bool) -> bool {
        let matches = self.items.iter().any(|item| match item {
            Item::Range(first, last) => {
                (*first..=*last).contains(&ch)
                    || (ignore_case
                        && unicode::variants(ch)
                            .iter()
                            .any(|variant| (*first..=*last).contains(variant)))
            }
            Item::Named(name) => name.matches(ch),
        });
        matches != self.negated
    }
}

fn literal(chars: &[char], cursor: &mut usize, end: usize) -> Result<char, PatternError> {
    let ch = chars[*cursor];
    *cursor += 1;
    if ch == '\\' && *cursor < end {
        let escaped = chars[*cursor];
        if "ntrbe".contains(escaped) {
            return Err(PatternError::UnsupportedRegex(format!("[\\{escaped}]")));
        }
        if "\\]-^".contains(escaped) {
            *cursor += 1;
            return Ok(escaped);
        }
    }
    Ok(ch)
}

fn named(
    chars: &[char],
    cursor: &mut usize,
    end: usize,
    scan: &Scan,
) -> Result<Option<Named>, PatternError> {
    if chars.get(*cursor..*cursor + 2) == Some(&['[', ':'])
        && let Some(after) = scan.named_end(*cursor + 2, end + 1)
    {
        let name: String = chars[*cursor + 2..after - 2].iter().collect();
        let named = Named::parse(&name)?;
        *cursor = after;
        Ok(Some(named))
    } else {
        Ok(None)
    }
}
