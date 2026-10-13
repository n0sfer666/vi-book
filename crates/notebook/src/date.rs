#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    pub fn to_days(self) -> i64 {
        let y = if self.month <= 2 {
            self.year - 1
        } else {
            self.year
        } as i64;
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let m = self.month as i64;
        let d = self.day as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    pub fn from_days(days: i64) -> Self {
        let z = days + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = if m <= 2 { y + 1 } else { y };
        Self {
            year: year as i32,
            month: m as u32,
            day: d as u32,
        }
    }

    pub fn succ(self) -> Self {
        Self::from_days(self.to_days() + 1)
    }

    pub fn pred(self) -> Self {
        Self::from_days(self.to_days() - 1)
    }

    pub fn weekday(self) -> &'static str {
        let idx = (self.to_days() + 3).rem_euclid(7);
        WEEKDAYS[idx as usize]
    }

    pub fn format(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    pub fn parse(text: &str) -> Option<Self> {
        let parts: Vec<&str> = text.split('-').collect();
        if let [y, m, d] = parts[..] {
            let year = y.parse().ok()?;
            let month = m.parse().ok()?;
            let day = d.parse().ok()?;
            if (1..=12).contains(&month) && (1..=31).contains(&day) {
                let date = Self::new(year, month, day);
                if date.is_valid() {
                    return Some(date);
                }
            }
        }
        None
    }

    fn is_valid(self) -> bool {
        Self::from_days(self.to_days()) == self
    }
}
