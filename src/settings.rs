use std::cell::Cell;
use std::fs;
use std::path::PathBuf;

use crate::db::Habit;

pub struct Settings {
    last_active_habit_id: Cell<i64>,
}

impl Clone for Settings {
    fn clone(&self) -> Self {
        Settings {
            last_active_habit_id: Cell::new(self.last_active_habit_id.get()),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            last_active_habit_id: Cell::new(-1),
        }
    }
}

impl Settings {
    fn settings_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/andrew".to_string());
        PathBuf::from(format!("{}/.pomotasker/pomotasker_settings.txt", home))
    }

    pub fn load() -> Self {
        let path = Self::settings_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            let id: i64 = contents.trim().parse().unwrap_or(-1);
            return Settings {
                last_active_habit_id: Cell::new(id),
            };
        }
        Settings::default()
    }

    pub fn save_active_habit(&self, habit: Option<Habit>) {
        if let Some(h) = habit {
            self.last_active_habit_id.set(h.id);
            let _ = fs::write(Self::settings_path(), h.id.to_string());
        } else {
            self.last_active_habit_id.set(-1);
            let _ = fs::write(Self::settings_path(), "");
        }
    }
}
