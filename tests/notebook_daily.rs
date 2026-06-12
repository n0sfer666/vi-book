use vi_book::notebook::daily::{DEFAULT_TEMPLATE, daily_path, expand_template, parse_daily_path};
use vi_book::notebook::date::Date;

#[test]
fn known_weekdays() {
    assert_eq!(Date::new(2026, 6, 12).weekday(), "Friday");
    assert_eq!(Date::new(2000, 1, 1).weekday(), "Saturday");
    assert_eq!(Date::new(1970, 1, 1).weekday(), "Thursday");
    assert_eq!(Date::new(2024, 2, 29).weekday(), "Thursday");
}

#[test]
fn succ_crosses_month_boundary() {
    assert_eq!(Date::new(2026, 1, 31).succ(), Date::new(2026, 2, 1));
}

#[test]
fn succ_crosses_year_boundary() {
    assert_eq!(Date::new(2026, 12, 31).succ(), Date::new(2027, 1, 1));
}

#[test]
fn pred_crosses_year_boundary() {
    assert_eq!(Date::new(2027, 1, 1).pred(), Date::new(2026, 12, 31));
}

#[test]
fn leap_year_february_has_29_days() {
    assert_eq!(Date::new(2024, 2, 28).succ(), Date::new(2024, 2, 29));
    assert_eq!(Date::new(2024, 2, 29).succ(), Date::new(2024, 3, 1));
}

#[test]
fn non_leap_year_february_has_28_days() {
    assert_eq!(Date::new(2026, 2, 28).succ(), Date::new(2026, 3, 1));
}

#[test]
fn century_non_leap_and_400_leap() {
    assert_eq!(Date::new(1900, 2, 28).succ(), Date::new(1900, 3, 1));
    assert_eq!(Date::new(2000, 2, 28).succ(), Date::new(2000, 2, 29));
}

#[test]
fn format_pads_components() {
    assert_eq!(Date::new(2026, 6, 5).format(), "2026-06-05");
    assert_eq!(Date::new(789, 12, 25).format(), "0789-12-25");
}

#[test]
fn parse_valid_date_roundtrips() {
    assert_eq!(Date::parse("2026-06-12"), Some(Date::new(2026, 6, 12)));
    assert_eq!(Date::parse("2024-02-29"), Some(Date::new(2024, 2, 29)));
}

#[test]
fn parse_rejects_invalid_dates() {
    assert_eq!(Date::parse("2026-13-01"), None);
    assert_eq!(Date::parse("2026-02-30"), None);
    assert_eq!(Date::parse("not-a-date"), None);
    assert_eq!(Date::parse("2026-06"), None);
}

#[test]
fn pred_succ_are_inverse() {
    let d = Date::new(2026, 3, 1);
    assert_eq!(d.pred().succ(), d);
    assert_eq!(d.succ().pred(), d);
}

#[test]
fn template_expands_date_and_weekday() {
    let out = expand_template("# {{date}} ({{weekday}})\n", Date::new(2026, 6, 12));
    assert_eq!(out, "# 2026-06-12 (Friday)\n");
}

#[test]
fn default_template_has_date_heading() {
    assert_eq!(
        expand_template(DEFAULT_TEMPLATE, Date::new(2026, 6, 12)),
        "# 2026-06-12\n\n"
    );
}

#[test]
fn daily_path_uses_daily_folder_and_iso_name() {
    assert_eq!(daily_path(Date::new(2026, 6, 12)), "daily/2026-06-12.md");
}

#[test]
fn parse_daily_path_extracts_date() {
    assert_eq!(
        parse_daily_path("daily/2026-06-12.md"),
        Some(Date::new(2026, 6, 12))
    );
    assert_eq!(parse_daily_path("notes/other.md"), None);
    assert_eq!(parse_daily_path("daily/not-a-date.md"), None);
}
