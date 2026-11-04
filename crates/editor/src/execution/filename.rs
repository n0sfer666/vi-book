use super::Error;

pub(super) fn parse(input: &str) -> Result<String, Error> {
    if input.starts_with(['%', '#'])
        || ["<afile>", "<amatch>", "<abuf>"]
            .iter()
            .any(|token| input.contains(token))
    {
        return Err(Error::Ex(
            "filename expansions are not supported yet".into(),
        ));
    }
    let mut out = String::new();
    let mut escape = false;
    for ch in input.chars() {
        if escape {
            out.push(ch);
            escape = false;
        } else if ch == '\\' {
            escape = true;
        } else if matches!(ch, ' ' | '\t') {
            return Err(Error::Ex("E172: Only one file name allowed".into()));
        } else {
            out.push(ch);
        }
    }
    if escape {
        out.push('\\');
    }
    Ok(out)
}
