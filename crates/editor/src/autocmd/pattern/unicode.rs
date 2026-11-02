use super::unicode_data::{FOLD, LOWER, PRINT, UPPER, VARIANTS};

pub(super) fn fold(ch: char) -> char {
    FOLD.binary_search_by_key(&ch, |&(key, _)| key)
        .map(|index| FOLD[index].1)
        .unwrap_or(ch)
}

pub(super) fn variants(ch: char) -> &'static [char] {
    VARIANTS
        .binary_search_by_key(&fold(ch), |&(key, _)| key)
        .map(|index| VARIANTS[index].1)
        .unwrap_or_default()
}

pub(super) fn lower(ch: char) -> bool {
    contains(LOWER, ch)
}

pub(super) fn upper(ch: char) -> bool {
    contains(UPPER, ch)
}

pub(super) fn printable(ch: char) -> bool {
    contains(PRINT, ch)
}

fn contains(ranges: &[(u32, u32)], ch: char) -> bool {
    let code = u32::from(ch);
    let index = ranges.partition_point(|&(start, _)| start <= code);
    index > 0 && code <= ranges[index - 1].1
}
