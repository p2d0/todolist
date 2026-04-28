use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Habit {
    pub id: i64,
    pub description: String,
    pub order_index: i32,
    pub timer_duration_seconds: u32, // 0 = stopwatch mode
    pub mode: HabitMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HabitMode {
    Timed,     // countdown timer
    Stopwatch, // elapsed timer
}

pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            conn: Connection::open(&self.path).expect("Failed to re-open database"),
            path: self.path.clone(),
        }
    }
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let path_buf = PathBuf::from(path);
        let conn = Connection::open(&path_buf)?;
        let db = Database {
            conn,
            path: path_buf,
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS habits (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                description TEXT NOT NULL,
                order_index INTEGER NOT NULL,
                timer_duration_seconds INTEGER NOT NULL DEFAULT 0,
                mode TEXT NOT NULL DEFAULT 'stopwatch'
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                habit_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                duration_seconds INTEGER NOT NULL,
                completed_at TEXT NOT NULL,
                FOREIGN KEY (habit_id) REFERENCES habits(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_habit_date ON sessions(habit_id, date);
            ",
        )?;
        Ok(())
    }

    // --- Habit CRUD ---

    pub fn add_habit(
        &self,
        description: &str,
        timer_duration_seconds: u32,
        mode: HabitMode,
    ) -> Result<Habit> {
        let max_order = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(order_index), -1) + 1 FROM habits",
                [],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0);

        let mode_str = match mode {
            HabitMode::Timed => "timed",
            HabitMode::Stopwatch => "stopwatch",
        };

        self.conn.execute(
            "INSERT INTO habits (description, order_index, timer_duration_seconds, mode) VALUES (?, ?, ?, ?)",
            params![description, max_order, timer_duration_seconds, mode_str],
        )?;

        self.get_habit(self.conn.last_insert_rowid())
    }

    pub fn get_habit(&self, id: i64) -> Result<Habit> {
        self.conn.query_row(
            "SELECT id, description, order_index, timer_duration_seconds, mode FROM habits WHERE id = ?",
            params![id],
            |row| Ok(Habit {
                id: row.get(0)?,
                description: row.get(1)?,
                order_index: row.get(2)?,
                timer_duration_seconds: row.get(3)?,
                mode: Self::parse_mode(row.get(4)?),
            }),
        )
    }

    pub fn get_all_habits(&self) -> Result<Vec<Habit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, description, order_index, timer_duration_seconds, mode FROM habits ORDER BY order_index ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Habit {
                id: row.get(0)?,
                description: row.get(1)?,
                order_index: row.get(2)?,
                timer_duration_seconds: row.get(3)?,
                mode: Self::parse_mode(row.get(4)?),
            })
        })?;
        rows.collect()
    }

    pub fn update_habit(
        &self,
        id: i64,
        description: &str,
        timer_duration_seconds: u32,
        mode: HabitMode,
    ) -> Result<()> {
        let mode_str = match mode {
            HabitMode::Timed => "timed",
            HabitMode::Stopwatch => "stopwatch",
        };

        self.conn.execute(
            "UPDATE habits SET description = ?, timer_duration_seconds = ?, mode = ? WHERE id = ?",
            params![description, timer_duration_seconds, mode_str, id],
        )?;
        Ok(())
    }

    pub fn delete_habit(&mut self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM habits WHERE id = ?", params![id])?;
        // Reorder remaining habits
        self.reorder_habits()?;
        Ok(())
    }

    pub fn reorder_habits(&mut self) -> Result<()> {
        // Reset order indices sequentially
        let habits: Vec<i64> = self
            .conn
            .prepare("SELECT id FROM habits ORDER BY order_index ASC")?
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>>>()?;

        for (i, id) in habits.iter().enumerate() {
            self.conn.execute(
                "UPDATE habits SET order_index = ? WHERE id = ?",
                params![i as i32, id],
            )?;
        }
        Ok(())
    }

    pub fn move_habit(&mut self, id: i64, new_index: i32) -> Result<()> {
        let all_habits: Vec<Habit> = self.get_all_habits()?;
        if new_index >= all_habits.len() as i32 || new_index < 0 {
            return Ok(());
        }

        let old_index = all_habits.iter().position(|h| h.id == id).unwrap() as i32;
        if old_index == new_index {
            return Ok(());
        }

        if old_index < new_index {
            // Moving down: shift intermediates up (decrement) to fill gap
            self.conn.execute(
                "UPDATE habits SET order_index = order_index - 1 WHERE id != ? AND order_index > ? AND order_index <= ?",
                params![id, old_index, new_index],
            )?;
        } else {
            // Moving up: shift intermediates down (increment) to fill gap
            self.conn.execute(
                "UPDATE habits SET order_index = order_index + 1 WHERE id != ? AND order_index >= ? AND order_index < ?",
                params![id, new_index, old_index],
            )?;
        }
        self.conn.execute(
            "UPDATE habits SET order_index = ? WHERE id = ?",
            params![new_index, id],
        )?;
        Ok(())
    }

    // --- Sessions ---

    pub fn insert_session(
        &self,
        habit_id: i64,
        date: NaiveDate,
        duration_seconds: u32,
        completed_at: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (habit_id, date, duration_seconds, completed_at) VALUES (?, ?, ?, ?)",
            params![habit_id, date.format("%Y-%m-%d").to_string(), duration_seconds, completed_at],
        )?;
        Ok(())
    }

    pub fn get_sessions_for_habit_date(&self, habit_id: i64, date: NaiveDate) -> Result<Vec<i64>> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM sessions WHERE habit_id = ? AND date = ?")?;
        let rows = stmt.query_map(params![habit_id, date_str], |row| row.get(0))?;
        rows.collect()
    }

    pub fn has_session_for_date(&self, habit_id: i64, date: NaiveDate) -> Result<bool> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE habit_id = ? AND date = ?",
                params![habit_id, date_str],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count > 0)
    }

    pub fn delete_session(&self, session_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM sessions WHERE id = ?", params![session_id])?;
        Ok(())
    }

    pub fn get_streak_days(&self, habit_id: i64) -> Result<u32> {
        // Count consecutive days ending with today that have at least one session
        let today = chrono::Local::now().date_naive();
        let mut streak = 0u32;

        for i in 0..365 {
            let date = today - chrono::Duration::days(i as i64);
            let date_str = date.format("%Y-%m-%d").to_string();
            let count: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM sessions WHERE habit_id = ? AND date = ?",
                    params![habit_id, date_str],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if count > 0 {
                streak += 1;
            } else {
                break;
            }
        }
        Ok(streak)
    }

    // --- Helpers ---

    fn parse_mode(s: String) -> HabitMode {
        match s.as_str() {
            "timed" => HabitMode::Timed,
            _ => HabitMode::Stopwatch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        Database::new(":memory:").expect("failed to create test db")
    }

    #[test]
    fn add_and_get_habit() {
        let db = test_db();
        let habit = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        assert_eq!(habit.description, "Read");
        assert_eq!(habit.timer_duration_seconds, 1500);
        assert_eq!(habit.mode, HabitMode::Timed);
        assert_eq!(habit.order_index, 0);

        let h = db.get_habit(habit.id).unwrap();
        assert_eq!(h.id, habit.id);
        assert_eq!(h.description, "Read");
    }

    #[test]
    fn add_multiple_habits_order() {
        let db = test_db();
        let h1 = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        let h2 = db.add_habit("Code", 900, HabitMode::Stopwatch).unwrap();
        let h3 = db.add_habit("Stretch", 0, HabitMode::Stopwatch).unwrap();

        assert_eq!(h1.order_index, 0);
        assert_eq!(h2.order_index, 1);
        assert_eq!(h3.order_index, 2);
    }

    #[test]
    fn get_all_habits_ordered() {
        let db = test_db();
        db.add_habit("C", 0, HabitMode::Stopwatch).unwrap();
        db.add_habit("A", 0, HabitMode::Stopwatch).unwrap();
        db.add_habit("B", 0, HabitMode::Stopwatch).unwrap();

        let all = db.get_all_habits().unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].description, "C");
        assert_eq!(all[1].description, "A");
        assert_eq!(all[2].description, "B");
    }

    #[test]
    fn update_habit() {
        let db = test_db();
        let h = db.add_habit("Old", 0, HabitMode::Stopwatch).unwrap();
        db.update_habit(h.id, "New", 1500, HabitMode::Timed)
            .unwrap();

        let updated = db.get_habit(h.id).unwrap();
        assert_eq!(updated.description, "New");
        assert_eq!(updated.timer_duration_seconds, 1500);
        assert_eq!(updated.mode, HabitMode::Timed);
    }

    #[test]
    fn delete_habit_reorders() {
        let mut db = test_db();
        let h1 = db.add_habit("A", 0, HabitMode::Stopwatch).unwrap();
        let h2 = db.add_habit("B", 0, HabitMode::Stopwatch).unwrap();
        let h3 = db.add_habit("C", 0, HabitMode::Stopwatch).unwrap();

        db.delete_habit(h2.id).unwrap();

        let all = db.get_all_habits().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].description, "A");
        assert_eq!(all[0].order_index, 0);
        assert_eq!(all[1].description, "C");
        assert_eq!(all[1].order_index, 1);
    }

    #[test]
    fn insert_and_query_session() {
        let db = test_db();
        let h = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        db.insert_session(h.id, date, 1500, "2025-01-15 10:00:00")
            .unwrap();

        let sessions = db.get_sessions_for_habit_date(h.id, date).unwrap();
        assert_eq!(sessions.len(), 1);

        assert!(db.has_session_for_date(h.id, date).unwrap());
    }

    #[test]
    fn delete_session() {
        let db = test_db();
        let h = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        db.insert_session(h.id, date, 1500, "2025-01-15 10:00:00")
            .unwrap();

        let ids = db.get_sessions_for_habit_date(h.id, date).unwrap();
        db.delete_session(ids[0]).unwrap();

        let ids2 = db.get_sessions_for_habit_date(h.id, date).unwrap();
        assert_eq!(ids2.len(), 0);
    }

    #[test]
    fn has_session_false() {
        let db = test_db();
        let h = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        assert!(!db.has_session_for_date(h.id, date).unwrap());
    }

    #[test]
    fn multiple_sessions_same_day() {
        let db = test_db();
        let h = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        db.insert_session(h.id, date, 1500, "2025-01-15 10:00:00")
            .unwrap();
        db.insert_session(h.id, date, 1500, "2025-01-15 14:00:00")
            .unwrap();

        let ids = db.get_sessions_for_habit_date(h.id, date).unwrap();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn move_habit_down() {
        let mut db = test_db();
        let h0 = db.add_habit("A", 0, HabitMode::Stopwatch).unwrap();
        let h1 = db.add_habit("B", 0, HabitMode::Stopwatch).unwrap();
        let h2 = db.add_habit("C", 0, HabitMode::Stopwatch).unwrap();
        let h3 = db.add_habit("D", 0, HabitMode::Stopwatch).unwrap();

        db.move_habit(h1.id, 3).unwrap();

        let all = db.get_all_habits().unwrap();
        assert_eq!(all[0].description, "A");
        assert_eq!(all[1].description, "C");
        assert_eq!(all[2].description, "D");
        assert_eq!(all[3].description, "B");
    }

    #[test]
    fn move_habit_up() {
        let mut db = test_db();
        let h0 = db.add_habit("A", 0, HabitMode::Stopwatch).unwrap();
        let h1 = db.add_habit("B", 0, HabitMode::Stopwatch).unwrap();
        let h2 = db.add_habit("C", 0, HabitMode::Stopwatch).unwrap();
        let h3 = db.add_habit("D", 0, HabitMode::Stopwatch).unwrap();

        db.move_habit(h3.id, 1).unwrap();

        let all = db.get_all_habits().unwrap();
        assert_eq!(all[0].description, "A");
        assert_eq!(all[1].description, "D");
        assert_eq!(all[2].description, "B");
        assert_eq!(all[3].description, "C");
    }

    #[test]
    fn move_habit_same_index_noop() {
        let mut db = test_db();
        let h0 = db.add_habit("A", 0, HabitMode::Stopwatch).unwrap();
        let h1 = db.add_habit("B", 0, HabitMode::Stopwatch).unwrap();

        db.move_habit(h1.id, 1).unwrap();

        let all = db.get_all_habits().unwrap();
        assert_eq!(all[0].description, "A");
        assert_eq!(all[1].description, "B");
    }

    #[test]
    fn move_habit_out_of_bounds_noop() {
        let mut db = test_db();
        let h = db.add_habit("A", 0, HabitMode::Stopwatch).unwrap();

        db.move_habit(h.id, 99).unwrap();
        let all = db.get_all_habits().unwrap();
        assert_eq!(all[0].order_index, 0);
    }

    #[test]
    fn reorder_habits() {
        let mut db = test_db();
        db.add_habit("A", 0, HabitMode::Stopwatch).unwrap();
        db.add_habit("B", 0, HabitMode::Stopwatch).unwrap();
        db.add_habit("C", 0, HabitMode::Stopwatch).unwrap();

        // Manually mess up order
        db.conn
            .execute(
                "UPDATE habits SET order_index = 10 WHERE description = 'B'",
                [],
            )
            .unwrap();

        db.reorder_habits().unwrap();

        let all = db.get_all_habits().unwrap();
        assert_eq!(all[0].order_index, 0);
        assert_eq!(all[1].order_index, 1);
        assert_eq!(all[2].order_index, 2);
    }

    #[test]
    fn streak_today() {
        let db = test_db();
        let h = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        let today = chrono::Local::now().date_naive();
        db.insert_session(h.id, today, 1500, "2025-01-15 10:00:00")
            .unwrap();

        let streak = db.get_streak_days(h.id).unwrap();
        assert_eq!(streak, 1);
    }

    #[test]
    fn streak_consecutive_days() {
        let db = test_db();
        let h = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        let today = chrono::Local::now().date_naive();

        for i in 0..5 {
            let d = today - chrono::Duration::days(i as i64);
            db.insert_session(h.id, d, 1500, "2025-01-15 10:00:00")
                .unwrap();
        }

        let streak = db.get_streak_days(h.id).unwrap();
        assert_eq!(streak, 5);
    }

    #[test]
    fn streak_breaks_on_gap() {
        let db = test_db();
        let h = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        let today = chrono::Local::now().date_naive();

        // today, yesterday, 4 days ago (gap on 3 days ago)
        db.insert_session(h.id, today, 1500, "2025-01-15 10:00:00")
            .unwrap();
        db.insert_session(
            h.id,
            today - chrono::Duration::days(1),
            1500,
            "2025-01-15 10:00:00",
        )
        .unwrap();
        db.insert_session(
            h.id,
            today - chrono::Duration::days(4),
            1500,
            "2025-01-15 10:00:00",
        )
        .unwrap();

        let streak = db.get_streak_days(h.id).unwrap();
        assert_eq!(streak, 2);
    }

    #[test]
    fn streak_zero_no_sessions() {
        let db = test_db();
        let h = db.add_habit("Read", 1500, HabitMode::Timed).unwrap();
        let streak = db.get_streak_days(h.id).unwrap();
        assert_eq!(streak, 0);
    }

    #[test]
    fn clone_database_independent() {
        // Use temp file (not :memory:) so clone shares same DB
        let path = format!("/tmp/pomotasker_test_clone_{}", std::process::id());
        let _ = std::fs::remove_file(&path);
        let db = Database::new(&path).unwrap();
        let db2 = db.clone();

        let h1 = db.add_habit("A", 0, HabitMode::Stopwatch).unwrap();
        let h2 = db2.add_habit("B", 0, HabitMode::Stopwatch).unwrap();

        assert_eq!(h1.description, "A");
        assert_eq!(h2.description, "B");

        // Both see each other's changes
        let all1 = db.get_all_habits().unwrap();
        let all2 = db2.get_all_habits().unwrap();
        assert_eq!(all1.len(), 2);
        assert_eq!(all2.len(), 2);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn habit_mode_parsing() {
        assert_eq!(Database::parse_mode("timed".to_string()), HabitMode::Timed);
        assert_eq!(
            Database::parse_mode("stopwatch".to_string()),
            HabitMode::Stopwatch
        );
        assert_eq!(
            Database::parse_mode("unknown".to_string()),
            HabitMode::Stopwatch
        );
    }
}
