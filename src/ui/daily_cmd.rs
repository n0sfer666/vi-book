use std::fs;

use crate::notebook::daily::{DEFAULT_TEMPLATE, daily_path, expand_template, parse_daily_path};
use crate::notebook::date::Date;
use crate::ui::app::App;

pub fn open_today(app: &mut App) {
    open_daily(app, app.today);
}

pub fn open_prev(app: &mut App) {
    open_daily(app, base_date(app).pred());
}

pub fn open_next(app: &mut App) {
    open_daily(app, base_date(app).succ());
}

fn base_date(app: &App) -> Date {
    parse_daily_path(&app.path).unwrap_or(app.today)
}

fn open_daily(app: &mut App, date: Date) {
    let rel = daily_path(date);
    let abs = app.session.absolute(&rel);
    if !abs.exists() {
        if let Some(parent) = abs.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            app.message = format!("daily failed: {err}");
            return;
        }
        if let Err(err) = fs::write(&abs, expand_template(DEFAULT_TEMPLATE, date)) {
            app.message = format!("daily failed: {err}");
            return;
        }
        let _ = app.session.reindex();
    }
    app.open_rel(&rel);
}
