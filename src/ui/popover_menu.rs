use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::db::{Database, Habit};
use crate::settings::Settings;

use super::timer_banner;

pub type TimerSharedRef = Rc<RefCell<super::TimerShared>>;

pub fn attach_popover(
    circle: &gtk4::Button,
    habit: &Habit,
    db: RefCell<Database>,
    settings: RefCell<Settings>,
    habits: RefCell<Vec<Habit>>,
    habit_list: gtk4::Box,
    wsl: gtk4::Label,
    timer_label: gtk4::Label,
    timer_btn: gtk4::Button,
    timer_shared: TimerSharedRef,
    window: gtk4::ApplicationWindow,
) {
    let popover = gtk4::Popover::new();
    popover.set_position(gtk4::PositionType::Bottom);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);

    // "Start Pomodoro" (timed mode)
    let btn_timed = gtk4::Button::with_label("▶ Start Pomodoro");

    // "Start Stopwatch"
    let btn_stopwatch = gtk4::Button::with_label("⏱ Start Stopwatch");

    // "Toggle Manual"
    let btn_manual = gtk4::Button::with_label("✓ Toggle Manual");

    content.append(&btn_timed);
    content.append(&btn_stopwatch);
    content.append(&btn_manual);

    popover.set_child(Some(&content));
    circle.set_popover(Some(&popover));

    // Timed start
    {
        let db = db.clone();
        let settings = settings.clone();
        let timer_shared = timer_shared.clone();
        let timer_label = timer_label.clone();
        let timer_btn = timer_btn.clone();
        let habit = habit.clone();

        btn_timed.connect_clicked(move |_| {
            // Stop current if any
            let aid_val = {
                let s = timer_shared.borrow();
                let aid = s.active_habit_id.borrow();
                *aid
            };
            if aid_val.is_some() {
                timer_banner::stop(
                    &timer_label,
                    &timer_btn,
                    timer_shared.clone(),
                    &db,
                    &settings,
                );
            }
            timer_banner::start(
                &timer_label,
                &timer_btn,
                &habit,
                timer_shared.clone(),
                &settings,
            );
        });
    }

    // Stopwatch start
    {
        let db = db.clone();
        let settings = settings.clone();
        let timer_shared = timer_shared.clone();
        let timer_label = timer_label.clone();
        let timer_btn = timer_btn.clone();
        let habit = habit.clone();

        btn_stopwatch.connect_clicked(move |_| {
            let aid_val = {
                let s = timer_shared.borrow();
                let aid = s.active_habit_id.borrow();
                *aid
            };
            if aid_val.is_some() {
                timer_banner::stop(
                    &timer_label,
                    &timer_btn,
                    timer_shared.clone(),
                    &db,
                    &settings,
                );
            }
            timer_banner::start(
                &timer_label,
                &timer_btn,
                &habit,
                timer_shared.clone(),
                &settings,
            );
        });
    }

    // Toggle manual (just mark today)
    {
        let db = db.clone();
        let habits = habits.clone();
        let habit_list = habit_list.clone();
        let wsl = wsl.clone();
        let timer_shared = timer_shared.clone();
        let timer_label = timer_label.clone();
        let timer_btn = timer_btn.clone();
        let settings = settings.clone();
        let window = window.clone();
        let habit_id = habit.id;
        let habit_duration = habit.timer_duration_seconds;

        btn_manual.connect_clicked(move |_| {
            let date = chrono::Local::now().naive_local().date();
            let has_session = db
                .borrow()
                .has_session_for_date(habit_id, date)
                .unwrap_or(false);
            if has_session {
                if let Ok(sessions) = db.borrow().get_sessions_for_habit_date(habit_id, date) {
                    if let Some(s) = sessions.first() {
                        let _ = db.borrow().delete_session(*s);
                    }
                }
            } else {
                let now_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = db
                    .borrow()
                    .insert_session(habit_id, date, habit_duration, &now_str);
            }
            super::refresh_from_closures(
                &db,
                &habits,
                &habit_list,
                &wsl,
                timer_shared.clone(),
                &timer_label,
                &timer_btn,
                &settings,
                &window,
            );
        });
    }
}
