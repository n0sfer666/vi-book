use crate::notebook::date::Date;

pub const DEFAULT_TEMPLATE: &str = "# {{date}}\n\n";

pub fn expand_template(template: &str, date: Date) -> String {
    template
        .replace("{{date}}", &date.format())
        .replace("{{weekday}}", date.weekday())
}

pub fn daily_path(date: Date) -> String {
    format!("daily/{}.md", date.format())
}

pub fn parse_daily_path(path: &str) -> Option<Date> {
    let name = path.strip_prefix("daily/")?.strip_suffix(".md")?;
    Date::parse(name)
}
