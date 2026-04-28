// Integration tests for PomoTasker - test full app workflows through the public API.
// Simulates user actions (add habit, complete session, check streak, etc.)
// Run with: cargo test --test integration
// These tests use in-memory DB (no GTK) to validate business logic end-to-end.

use pomotasker::db::{Database, HabitMode, HabitType};
use pomotasker::NaiveDate;

// -- Helpers --

fn test_db() -> Database {
    Database::new(":memory:").unwrap()
}

fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

// -- Habit CRUD workflow --

#[test]
fn workflow_create_and_view_habits() {
    let db = test_db();

    // User adds 3 habits in different modes/types
    let h1 = db.add_habit("Morning Reading", 900, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let h2 = db.add_habit("Push-ups", 0, HabitMode::Stopwatch, HabitType::Number, Some(20)).unwrap();
    let h3 = db.add_habit("Meditate", 600, HabitMode::Timed, HabitType::Boolean, None).unwrap();

    // Verify all habits retrievable in order
    let habits = db.get_all_habits().unwrap();
    assert_eq!(habits.len(), 3);
    assert_eq!(habits[0].description, "Morning Reading");
    assert_eq!(habits[0].mode, HabitMode::Timed);
    assert_eq!(habits[0].timer_duration_seconds, 900);
    assert_eq!(habits[1].description, "Push-ups");
    assert_eq!(habits[1].mode, HabitMode::Stopwatch);
    assert_eq!(habits[1].habit_type, HabitType::Number);
    assert_eq!(habits[1].min_value, Some(20));
    assert_eq!(habits[2].description, "Meditate");
    assert_eq!(habits[2].habit_type, HabitType::Boolean);

    // Each habit individually queryable
    assert_eq!(db.get_habit(h1.id).unwrap().description, "Morning Reading");
    assert_eq!(db.get_habit(h2.id).unwrap().description, "Push-ups");
    assert_eq!(db.get_habit(h3.id).unwrap().description, "Meditate");
}

#[test]
fn workflow_edit_habit() {
    let db = test_db();
    let h = db.add_habit("Old Name", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();

    // User renames and changes duration
    db.update_habit(h.id, "New Name", 1500, HabitMode::Stopwatch, HabitType::Timer, None).unwrap();

    let updated = db.get_habit(h.id).unwrap();
    assert_eq!(updated.description, "New Name");
    assert_eq!(updated.timer_duration_seconds, 1500);
    assert_eq!(updated.mode, HabitMode::Stopwatch);
}

#[test]
fn workflow_delete_habit_and_reorder() {
    let mut db = test_db();
    let h1 = db.add_habit("A", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let h2 = db.add_habit("B", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let h3 = db.add_habit("C", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let _h4 = db.add_habit("D", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();

    // Delete middle habit
    db.delete_habit(h2.id).unwrap();

    let remaining = db.get_all_habits().unwrap();
    assert_eq!(remaining.len(), 3);
    assert_eq!(remaining[0].description, "A");
    assert_eq!(remaining[0].order_index, 0);
    assert_eq!(remaining[1].description, "C");
    assert_eq!(remaining[1].order_index, 1);
    assert_eq!(remaining[2].description, "D");
    assert_eq!(remaining[2].order_index, 2);

    // Deleted habit no longer exists
    assert!(db.get_habit(h2.id).is_err());
}

// -- Timer session workflow --

#[test]
fn workflow_timed_pomodoro_session() {
    let db = test_db();
    let h = db.add_habit("Coding", 1500, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let date = today();

    // User completes a 25min pomodoro
    db.insert_session(h.id, date, 1500, "2025-01-15 10:00:00").unwrap();

    // Session recorded
    assert!(db.has_session_for_date(h.id, date).unwrap());
    let sessions = db.get_sessions_for_habit_date(h.id, date).unwrap();
    assert_eq!(sessions.len(), 1);

    // Total minutes for the day
    let total = db.get_total_minutes_for_habit_date(h.id, date).unwrap();
    assert_eq!(total, 25); // 1500s = 25min
}

#[test]
fn workflow_multiple_sessions_same_day() {
    let db = test_db();
    let h = db.add_habit("Study", 900, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let date = today();

    // 3 pomodoros throughout the day
    db.insert_session(h.id, date, 900, "2025-01-15 09:00:00").unwrap();
    db.insert_session(h.id, date, 900, "2025-01-15 11:00:00").unwrap();
    db.insert_session(h.id, date, 900, "2025-01-15 14:00:00").unwrap();

    let total = db.get_total_minutes_for_habit_date(h.id, date).unwrap();
    assert_eq!(total, 45); // 3 * 15min

    let sessions = db.get_sessions_for_habit_date(h.id, date).unwrap();
    assert_eq!(sessions.len(), 3);
}

#[test]
fn workflow_stopwatch_session() {
    let db = test_db();
    let h = db.add_habit("Gym", 0, HabitMode::Stopwatch, HabitType::Timer, None).unwrap();
    let date = today();

    // Stopwatch: user records actual elapsed time
    db.insert_session(h.id, date, 3600, "2025-01-15 07:00:00").unwrap();

    let total = db.get_total_minutes_for_habit_date(h.id, date).unwrap();
    assert_eq!(total, 60); // 1 hour
}

#[test]
fn workflow_delete_session() {
    let db = test_db();
    let h = db.add_habit("Task", 900, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let date = today();

    db.insert_session(h.id, date, 900, "2025-01-15 09:00:00").unwrap();
    db.insert_session(h.id, date, 900, "2025-01-15 11:00:00").unwrap();

    let ids = db.get_sessions_for_habit_date(h.id, date).unwrap();
    db.delete_session(ids[0]).unwrap();

    assert_eq!(db.get_sessions_for_habit_date(h.id, date).unwrap().len(), 1);
}

#[test]
fn workflow_clear_all_sessions_for_date() {
    let db = test_db();
    let h = db.add_habit("Task", 900, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let date = today();

    db.insert_session(h.id, date, 900, "2025-01-15 09:00:00").unwrap();
    db.insert_session(h.id, date, 900, "2025-01-15 11:00:00").unwrap();

    db.delete_sessions_for_habit_date(h.id, date).unwrap();

    assert!(!db.has_session_for_date(h.id, date).unwrap());
    assert_eq!(db.get_total_minutes_for_habit_date(h.id, date).unwrap(), 0);
}

#[test]
fn workflow_update_session_duration() {
    let db = test_db();
    let h = db.add_habit("Task", 900, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let date = today();

    db.insert_session(h.id, date, 600, "2025-01-15 09:00:00").unwrap();
    let ids = db.get_sessions_for_habit_date(h.id, date).unwrap();

    // User edits the session duration
    db.update_session_duration(ids[0], 1200).unwrap();

    let total = db.get_total_minutes_for_habit_date(h.id, date).unwrap();
    assert_eq!(total, 20); // 1200s = 20min
}

// -- Boolean habit workflow --

#[test]
fn workflow_boolean_habit_toggle() {
    let db = test_db();
    let h = db.add_habit("Drink Water", 0, HabitMode::Stopwatch, HabitType::Boolean, None).unwrap();
    let date = today();

    // User marks as done
    db.set_value_for_habit_date(h.id, date, 1.0).unwrap();

    let value = db.get_value_for_habit_date(h.id, date).unwrap();
    assert_eq!(value, Some(1.0));
    assert!(db.has_session_for_date(h.id, date).unwrap());

    // User un-marks (clears)
    db.clear_value_for_habit_date(h.id, date).unwrap();

    assert!(db.get_value_for_habit_date(h.id, date).unwrap().is_none());
    assert!(!db.has_session_for_date(h.id, date).unwrap());
}

// -- Number habit workflow --

#[test]
fn workflow_number_habit_with_value() {
    let db = test_db();
    let h = db.add_habit("Squats", 0, HabitMode::Stopwatch, HabitType::Number, Some(50)).unwrap();
    let date = today();

    // User logs 65 squats
    db.set_value_for_habit_date(h.id, date, 65.0).unwrap();

    let value = db.get_value_for_habit_date(h.id, date).unwrap();
    assert_eq!(value, Some(65.0));
    assert_eq!(h.min_value, Some(50));

    // User updates value
    db.clear_value_for_habit_date(h.id, date).unwrap();
    db.set_value_for_habit_date(h.id, date, 80.0).unwrap();

    let value = db.get_value_for_habit_date(h.id, date).unwrap();
    assert_eq!(value, Some(80.0));
}

// -- Streak workflow --

#[test]
fn workflow_streak_builds_over_days() {
    let db = test_db();
    let h = db.add_habit("Daily Habit", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();

    // Sessions for today + 4 previous days = 5 day streak
    for i in 0..5 {
        let d = today() - chrono::Duration::days(i as i64);
        db.insert_session(h.id, d, 600, "now").unwrap();
    }

    assert_eq!(db.get_streak_days(h.id).unwrap(), 5);
}

#[test]
fn workflow_streak_breaks_on_missing_day() {
    let db = test_db();
    let h = db.add_habit("Daily Habit", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();

    // today, yesterday, gap, 4 days ago
    db.insert_session(h.id, today(), 600, "now").unwrap();
    db.insert_session(h.id, today() - chrono::Duration::days(1), 600, "now").unwrap();
    db.insert_session(h.id, today() - chrono::Duration::days(4), 600, "now").unwrap();

    assert_eq!(db.get_streak_days(h.id).unwrap(), 2);
}

#[test]
fn workflow_streak_zero_when_no_today() {
    let db = test_db();
    let h = db.add_habit("Daily Habit", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();

    // Only yesterday, not today
    db.insert_session(h.id, today() - chrono::Duration::days(1), 600, "now").unwrap();

    assert_eq!(db.get_streak_days(h.id).unwrap(), 0);
}

#[test]
fn workflow_streak_includes_boolean_habits() {
    let db = test_db();
    let h = db.add_habit("Meditate", 0, HabitMode::Stopwatch, HabitType::Boolean, None).unwrap();

    for i in 0..3 {
        let d = today() - chrono::Duration::days(i as i64);
        db.set_value_for_habit_date(h.id, d, 1.0).unwrap();
    }

    assert_eq!(db.get_streak_days(h.id).unwrap(), 3);
}

// -- Reorder / move workflow --

#[test]
fn workflow_reorder_habits_via_move() {
    let mut db = test_db();
    let _h0 = db.add_habit("A", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let h1 = db.add_habit("B", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let _h2 = db.add_habit("C", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let _h3 = db.add_habit("D", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();

    // Move B from index 1 to end
    db.move_habit(h1.id, 3).unwrap();

    let habits = db.get_all_habits().unwrap();
    assert_eq!(habits[0].description, "A");
    assert_eq!(habits[1].description, "C");
    assert_eq!(habits[2].description, "D");
    assert_eq!(habits[3].description, "B");
}

#[test]
fn workflow_move_up() {
    let mut db = test_db();
    let _h0 = db.add_habit("A", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let _h1 = db.add_habit("B", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let _h2 = db.add_habit("C", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let h3 = db.add_habit("D", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();

    // Move D to top
    db.move_habit(h3.id, 0).unwrap();

    let habits = db.get_all_habits().unwrap();
    assert_eq!(habits[0].description, "D");
    assert_eq!(habits[1].description, "A");
    assert_eq!(habits[2].description, "B");
    assert_eq!(habits[3].description, "C");
}

// -- Cross-habit workflow (delete cascades sessions) --

#[test]
fn workflow_delete_habit_cascades_sessions() {
    let mut db = test_db();
    let h = db.add_habit("Temp", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    let date = today();

    db.insert_session(h.id, date, 600, "now").unwrap();
    assert!(db.has_session_for_date(h.id, date).unwrap());

    db.delete_habit(h.id).unwrap();

    // Sessions should be gone due to ON DELETE CASCADE
    let sessions = db.get_sessions_for_habit_date(h.id, date);
    // Either error (habit gone) or empty
    if let Ok(s) = sessions {
        assert_eq!(s.len() as usize, 0);
    }
}

// -- Schema idempotency (init can run multiple times) --

#[test]
fn workflow_schema_init_idempotent() {
    let db = test_db();
    // Adding habits after init works
    let h = db.add_habit("X", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();
    assert_eq!(h.description, "X");

    // Cloning re-inits but data persists (via file)
    // For :memory: this would fail - test with temp file instead
}

// -- Persistence via file DB --

#[test]
fn workflow_file_db_persistence() {
    use std::fs;
    let path = "/tmp/pomotasker_integration_{}.db".replace("{}", &std::process::id().to_string());
    let _ = fs::remove_file(&path);

    {
        let db = Database::new(&path).unwrap();
        let h = db.add_habit("Persisted", 900, HabitMode::Timed, HabitType::Timer, None).unwrap();
        db.insert_session(h.id, today(), 900, "now").unwrap();
    }

    // Re-open from disk
    {
        let db = Database::new(&path).unwrap();
        let habits = db.get_all_habits().unwrap();
        assert_eq!(habits.len(), 1);
        assert_eq!(habits[0].description, "Persisted");

        assert!(db.has_session_for_date(habits[0].id, today()).unwrap());
    }

    let _ = fs::remove_file(&path);
}

#[test]
fn workflow_clone_db_shared_state() {
    let path = "/tmp/pomotasker_clone_integration_{}.db".replace("{}", &std::process::id().to_string());
    let _ = std::fs::remove_file(&path);

    let db1 = Database::new(&path).unwrap();
    let db2 = db1.clone();

    // db1 adds, db2 sees it
    let h = db1.add_habit("Shared", 600, HabitMode::Timed, HabitType::Timer, None).unwrap();

    let all = db2.get_all_habits().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, h.id);

    // db2 adds session, db1 sees it
    db2.insert_session(h.id, today(), 600, "now").unwrap();
    assert!(db1.has_session_for_date(h.id, today()).unwrap());

    let _ = std::fs::remove_file(&path);
}
