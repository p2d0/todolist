use std::cell::RefCell;

use chrono::{Datelike, NaiveDate};

use crate::db::{Database, Habit, HabitMode, HabitType};

/// Business logic layer — all habit/session operations go through here.
/// Keeps UI code free of direct DB queries.
pub struct AppState {
    pub db: RefCell<Database>,
    pub habits: RefCell<Vec<Habit>>,
}

impl AppState {
    pub fn new(db: &Database) -> Self {
        let habits = RefCell::new(db.get_all_habits().unwrap_or_default());
        AppState {
            db: RefCell::new(db.clone()),
            habits,
        }
    }

    pub fn reload_habits(&self) {
        *self.habits.borrow_mut() = self.db.borrow().get_all_habits().unwrap_or_default();
    }

    pub fn add_habit(
        &self,
        description: &str,
        timer_duration_seconds: u32,
        mode: HabitMode,
        habit_type: HabitType,
        min_value: Option<u32>,
    ) {
        let _ = self.db.borrow_mut().add_habit(
            description,
            timer_duration_seconds,
            mode,
            habit_type,
            min_value,
        );
        self.reload_habits();
    }

    pub fn update_habit(
        &self,
        habit: &Habit,
        new_description: &str,
        new_timer_duration: u32,
        new_mode: HabitMode,
        new_habit_type: HabitType,
        new_min_value: Option<u32>,
    ) {
        let _ = self.db.borrow_mut().update_habit(
            habit.id,
            new_description,
            new_timer_duration,
            new_mode,
            new_habit_type,
            new_min_value,
        );
        self.reload_habits();
    }

    pub fn delete_habit(&self, id: i64) {
        let _ = self.db.borrow_mut().delete_habit(id);
        self.reload_habits();
    }

    pub fn add_session_for_date(&self, habit_id: i64, date: NaiveDate, duration_seconds: u32) {
        let now_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = self.db.borrow().insert_session(habit_id, date, duration_seconds, &now_str);
    }

    pub fn set_session_minutes_for_date(&self, habit_id: i64, date: NaiveDate, minutes: u32) {
        let seconds = minutes * 60;
        let _ = self.db.borrow_mut().delete_sessions_for_habit_date(habit_id, date);
        if seconds > 0 {
            let now_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let _ = self.db.borrow_mut().insert_session(habit_id, date, seconds, &now_str);
        }
    }

    pub fn remove_session_for_date(&self, habit_id: i64, date: NaiveDate) {
        let _ = self.db.borrow().delete_sessions_for_habit_date(habit_id, date);
    }

    pub fn get_total_minutes(&self, habit_id: i64, date: NaiveDate) -> u32 {
        self.db.borrow().get_total_minutes_for_habit_date(habit_id, date).unwrap_or(0)
    }

    pub fn has_session(&self, habit_id: i64, date: NaiveDate) -> bool {
        self.db.borrow().has_session_for_date(habit_id, date).unwrap_or(false)
    }

    pub fn set_number_value(&self, habit_id: i64, date: NaiveDate, value: f64) {
        let _ = self.db.borrow_mut().delete_sessions_for_habit_date(habit_id, date);
        let _ = self.db.borrow_mut().set_value_for_habit_date(habit_id, date, value);
    }

    pub fn get_number_value(&self, habit_id: i64, date: NaiveDate) -> Option<f64> {
        self.db.borrow().get_value_for_habit_date(habit_id, date).unwrap_or(None)
    }

    pub fn clear_number_value(&self, habit_id: i64, date: NaiveDate) {
        let _ = self.db.borrow().clear_value_for_habit_date(habit_id, date);
    }

    pub fn get_circle_label(&self, habit: &Habit, date: NaiveDate) -> String {
        match habit.habit_type {
            HabitType::Timer => {
                let mins = self.get_total_minutes(habit.id, date);
                if mins > 0 {
                    format!("{}m", mins)
                } else {
                    day_letter(date.weekday().number_from_monday() as i64)
                }
            }
            HabitType::Boolean => {
                let val = self.db.borrow().get_value_for_habit_date(habit.id, date).unwrap_or(None);
                if val.is_some() {
                    "✓".to_string()
                } else {
                    day_letter(date.weekday().number_from_monday() as i64)
                }
            }
            HabitType::Number => {
                let val = self.get_number_value(habit.id, date);
                match (val, habit.min_value) {
                    (Some(v), _) => v.to_string(),
                    (None, _) => day_letter(date.weekday().number_from_monday() as i64),
                }
            }
        }
    }
}

fn day_letter(dow: i64) -> String {
    match dow {
        1 => "M".into(),
        2 => "T".into(),
        3 => "W".into(),
        4 => "T".into(),
        5 => "F".into(),
        6 => "S".into(),
        7 => "S".into(),
        _ => "?".into(),
    }
}
