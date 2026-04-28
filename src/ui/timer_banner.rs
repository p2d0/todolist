use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::db::Habit;
use crate::settings::Settings;

pub type TimerSharedRef = Rc<RefCell<crate::ui::TimerShared>>;

pub fn start(
    label: &gtk4::Label,
    btn: &gtk4::Button,
    habit: &Habit,
    shared: TimerSharedRef,
    settings: &RefCell<Settings>,
) {
    shared.borrow_mut().active_habit_id.replace(Some(habit.id));
    shared
        .borrow_mut()
        .timer_start_instant
        .replace(Some(std::time::Instant::now()));
    shared.borrow_mut().timer_elapsed_before.replace(0);
    btn.set_label("Stop");
    btn.set_css_classes(&["timer-banner-button", "active"]);
    label.set_css_classes(&["timer-banner-label", "active"]);
    shared.borrow().clock.set_progress(0.0);
    settings.borrow().save_active_habit(Some(habit.clone()));
}

pub fn stop(
    label: &gtk4::Label,
    btn: &gtk4::Button,
    shared: TimerSharedRef,
    db: &RefCell<crate::db::Database>,
    settings: &RefCell<Settings>,
) {
    let (id, start_opt, elapsed_before) = {
        let s = match shared.try_borrow_mut() {
            Ok(s) => s,
            Err(_) => return,
        };
        let id = s.active_habit_id.replace(None);
        let start_opt = s.timer_start_instant.replace(None);
        let elapsed_before = s.timer_elapsed_before.replace(0);
        (id, start_opt, elapsed_before)
    };

    if let Some(id) = id {
        let elapsed = match start_opt {
            Some(start) => elapsed_before + start.elapsed().as_secs() as u32,
            None => elapsed_before,
        };

        let date = chrono::Local::now().naive_local().date();
        let now_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = db.borrow().insert_session(id, date, elapsed, &now_str);

        settings.borrow().save_active_habit(None);
    }

    {
        let s = shared.try_borrow_mut().expect("stop: borrow source_id");
        s.timer_source_id.replace(None);
    }
    btn.set_label("Start");
    btn.set_css_classes(&["timer-banner-button"]);
    label.set_label("0.0 pomodoros");
    label.set_css_classes(&["timer-banner-label"]);
    shared.borrow().clock.set_progress(0.0);
}

pub fn update_tick(
    label: &gtk4::Label,
    btn: &gtk4::Button,
    habit: &Habit,
    shared: TimerSharedRef,
    db: &RefCell<crate::db::Database>,
    settings: &RefCell<Settings>,
) {
    let (active, start_opt, elapsed_before) = {
        let s = shared.borrow();
        let active = s.active_habit_id.borrow().is_some();
        let start_opt = *s.timer_start_instant.borrow();
        let elapsed_before = *s.timer_elapsed_before.borrow();
        (active, start_opt, elapsed_before)
    };

    if !active {
        shared.borrow_mut().timer_source_id.replace(None);
        return;
    }

    let elapsed = match start_opt {
        Some(start) => elapsed_before + start.elapsed().as_secs() as u32,
        None => elapsed_before,
    };

    if habit.mode == crate::db::HabitMode::Timed && elapsed >= habit.timer_duration_seconds {
        stop(label, btn, shared.clone(), db, settings);
        return;
    }

    update_display(label, btn, habit, elapsed, &shared);
}

pub fn update_display(
    label: &gtk4::Label,
    btn: &gtk4::Button,
    habit: &Habit,
    elapsed: u32,
    shared: &TimerSharedRef,
) {
    let clock = shared.borrow().clock.clone();
    if habit.mode == crate::db::HabitMode::Timed {
        let remaining = habit.timer_duration_seconds.saturating_sub(elapsed);
        let pomos = elapsed as f64 / habit.timer_duration_seconds as f64;
        let text = if remaining > 0 {
            format!("{:.1} pomodoros ({}s left)", pomos, remaining)
        } else {
            format!("{:.1} pomodoros (complete!)", pomos)
        };
        label.set_label(&text);
        label.set_css_classes(&["timer-banner-label", "active"]);
        btn.set_css_classes(&["timer-banner-button", "active"]);
        clock.set_progress(pomos);
    } else {
        // Stopwatch mode: show elapsed time
        clock.set_stopwatch_elapsed(elapsed);
        label.set_css_classes(&["timer-banner-label", "active"]);
        btn.set_css_classes(&["timer-banner-button", "active"]);
    }
}
