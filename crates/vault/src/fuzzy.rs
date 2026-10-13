fn fold(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

pub fn score(query: &str, candidate: &str) -> Option<i32> {
    let q: Vec<char> = query.chars().map(fold).collect();
    if q.is_empty() {
        return Some(0);
    }
    let c: Vec<char> = candidate.chars().collect();
    let mut qi = 0;
    let mut total = 0;
    let mut prev_match: Option<usize> = None;
    for (i, &orig) in c.iter().enumerate() {
        let ch = fold(orig);
        if qi < q.len() && ch == q[qi] {
            let mut bonus = 0;
            if prev_match == Some(i.wrapping_sub(1)) {
                bonus += 10;
            }
            if i == 0 || is_boundary(c.get(i.wrapping_sub(1)).copied()) {
                bonus += 5;
            }
            total += bonus - i as i32;
            prev_match = Some(i);
            qi += 1;
        }
    }
    (qi == q.len()).then_some(total)
}

pub fn filter<'a>(query: &str, items: &'a [String]) -> Vec<&'a str> {
    let mut scored: Vec<(i32, usize, &str)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| score(query, item).map(|s| (s, i, item.as_str())))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, _, item)| item).collect()
}

fn is_boundary(prev: Option<char>) -> bool {
    matches!(prev, Some('/') | Some(' ') | Some('-') | Some('_'))
}
