use std::time::{SystemTime, UNIX_EPOCH};

use crate::notebook::date::Date;

pub fn today() -> Date {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    Date::from_days(secs.div_euclid(86_400))
}
