use std::cell::RefCell;

use chrono::{Datelike, Local};

use crate::db::{Habit, HabitType};

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
        let mut timer_minutes = 0u32;
        for i in 0i64..7 {
            let date = week_start + chrono::Duration::days(i);
            if let Ok(has) = db.borrow().has_session_for_date(habit.id, date) {
                if has {
                    sessions += 1;
                }
            }
            // Sum actual timer minutes for timer-type habits
            if habit.habit_type == HabitType::Timer {
                if let Ok(mins) = db.borrow().get_total_minutes_for_habit_date(habit.id, date) {
                    timer_minutes += mins;
                }
            }
        }
        total_sessions += sessions;

        // Convert actual minutes to pomodoros using habit's target duration
        if habit.habit_type == HabitType::Timer && habit.timer_duration_seconds > 0 && timer_minutes > 0 {
            let duration_mins = habit.timer_duration_seconds as f64 / 60.0;
            total_pomodoros += timer_minutes as f64 / duration_mins;
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
