use std::cell::RefCell;

use chrono::{Datelike, Local};

use crate::db::Habit;

pub fn update(
    db: &RefCell<crate::db::Database>,
    habits: &RefCell<Vec<Habit>>,
    label: &gtk4::Label,
) {
    let today = Local::now().date_naive();
    let today_dow = today.weekday().number_from_monday();
    let week_start = today - chrono::Duration::days((today_dow as u64 - 1) as i64);

    let mut total_sessions = 0u32;
    let mut total_pomodoros = 0f64;
    let mut max_streak = 0u32;

    for habit in &*habits.borrow() {
        let mut sessions = 0u32;
        for i in 0i64..7 {
            let date = week_start + chrono::Duration::days(i);
            if let Ok(has) = db.borrow().has_session_for_date(habit.id, date) {
                if has {
                    sessions += 1;
                }
            }
        }
        total_sessions += sessions;

        if habit.timer_duration_seconds > 0 {
            total_pomodoros += sessions as f64;
        }

        let streak = db.borrow().get_streak_days(habit.id).unwrap_or(0);
        if streak > max_streak {
            max_streak = streak;
        }
    }

    let week_label = format!(
        "📊 Week: {} sessions, {:.1} pomodoros, 🔥 {} day streak",
        total_sessions, total_pomodoros, max_streak
    );
    label.set_label(&week_label);
}
