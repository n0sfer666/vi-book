pub(super) fn settings(line: &str) -> Option<Vec<String>> {
    let (tail, uppercase) = marker(line)?;
    let tail = tail.trim_start_matches([' ', '\t']);
    if uppercase && !tail.starts_with("set ") {
        return None;
    }
    let mut settings = Vec::new();
    let mut part = String::new();
    for ch in tail.chars() {
        if ch == ':' && part.ends_with('\\') {
            part.pop();
            part.push(ch);
        } else if ch == ':' {
            if let Some(setting) = set_form(&part) {
                settings.push(setting.to_string());
                return Some(settings);
            }
            settings.push(std::mem::take(&mut part));
        } else {
            part.push(ch);
        }
    }
    if set_form(&part).is_none() {
        settings.push(part);
    }
    Some(settings)
}

fn set_form(part: &str) -> Option<&str> {
    let part = part.trim_start_matches([' ', '\t']);
    part.strip_prefix("set ")
        .or_else(|| part.strip_prefix("se "))
}

fn marker(line: &str) -> Option<(&str, bool)> {
    for (index, _) in line.char_indices() {
        if index > 0 && !matches!(line.as_bytes()[index - 1], b' ' | b'\t') {
            continue;
        }
        let rest = &line[index..];
        if index > 0
            && let Some(tail) = rest.strip_prefix("ex:")
        {
            return Some((tail, false));
        }
        let uppercase = rest.starts_with("Vim");
        let rest = if let Some(rest) = rest
            .strip_prefix("vim")
            .or_else(|| rest.strip_prefix("Vim"))
        {
            rest
        } else if let Some(tail) = rest.strip_prefix("vi:") {
            return Some((tail, false));
        } else {
            continue;
        };
        if let Some(tail) = rest.strip_prefix(':') {
            return Some((tail, uppercase));
        }
        let (relation, digits) = match rest.as_bytes().first() {
            Some(b'<' | b'=' | b'>') => (&rest[..1], &rest[1..]),
            _ => ("", rest),
        };
        let end = digits
            .find(|ch: char| !ch.is_ascii_digit())
            .unwrap_or(digits.len());
        let Some(tail) = digits[end..].strip_prefix(':') else {
            continue;
        };
        let Ok(version) = digits[..end].parse::<i32>() else {
            continue;
        };
        let matches = match relation {
            "<" => 801 < version,
            "=" => 801 == version,
            ">" => 801 > version,
            _ => 801 >= version,
        };
        if matches {
            return Some((tail, uppercase));
        }
    }
    None
}
