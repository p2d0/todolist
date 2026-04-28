use chrono::{Datelike, Local};
use gtk4::prelude::*;
use pango::EllipsizeMode;

use crate::db::Habit;

use super::timer_banner;
use super::RowContext;

pub fn create_row(ctx: &RowContext, habit: &Habit) {
    let row = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    row.add_css_class("habit-row");

    // Compact layout for narrow windows: desc + edit on top, day circles below
    let top_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let desc = gtk4::Label::new(Some(&habit.description));
    desc.set_ellipsize(EllipsizeMode::End);
    desc.set_hexpand(true);
    desc.set_selectable(true);
    desc.set_xalign(0.0);
    desc.add_css_class("habit-desc");

    // Day circles (Mon-Sun)
    let week_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    week_box.set_hexpand(true);
    week_box.set_valign(gtk4::Align::Center);

    let today = Local::now().date_naive();
    let today_dow = today.weekday().number_from_monday();
    let week_start = today - chrono::Duration::days((today_dow as u64 - 1) as i64);

    for i in 0i64..7 {
        let date = week_start + chrono::Duration::days(i);
        let day_num = date.weekday().number_from_monday() as i64;
        let day_label = match day_num {
            1 => "M",
            2 => "T",
            3 => "W",
            4 => "T",
            5 => "F",
            6 => "S",
            7 => "S",
            _ => "?",
        };

        let circle = gtk4::Button::with_label(day_label);
        circle.add_css_class("day-circle");

        let has_session = ctx
            .db
            .borrow()
            .has_session_for_date(habit.id, date)
            .unwrap_or(false);
        if has_session {
            circle.add_css_class("day-completed");
        }

        let is_today = i == (today_dow as i64 - 1);
        if is_today {
            circle.add_css_class("day-today");
        }

        // Restore day-active if this habit has active timer
        let is_active = {
            let s = ctx.timer_shared.borrow();
            let aid = s.active_habit_id.borrow();
            *aid == Some(habit.id)
        };
        if is_today && is_active {
            circle.add_css_class("day-active");
            circle.add_css_class("day-completed");
        }

        let db = ctx.db.clone();
        let habits = ctx.habits.clone();
        let habit_list = ctx.habit_list.clone();
        let wsl = ctx.wsl.clone();
        let timer_label = ctx.timer_label.clone();
        let timer_btn = ctx.timer_btn.clone();
        let timer_shared = ctx.timer_shared.clone();
        let settings = ctx.settings.clone();
        let window = ctx.window.clone();
        let habit = habit.clone();
        let habit_id = habit.id;
        let habit_duration = habit.timer_duration_seconds;
        let is_today = is_today;
        let has_session = has_session;
        let circle_c = circle.clone();

        circle.connect_clicked(move |_| {
            let aid_val = {
                let s = timer_shared.borrow();
                let aid = s.active_habit_id.borrow();
                *aid
            };
            if is_today {
                if aid_val == Some(habit_id) {
                    timer_banner::stop(
                        &timer_label,
                        &timer_btn,
                        timer_shared.clone(),
                        &db,
                        &settings,
                    );
                    circle_c.remove_css_class("day-completed");
                } else {
                    // Stop old habit first if switching
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
                    circle_c.add_css_class("day-active");
                    circle_c.add_css_class("day-completed");
                }
            } else {
                if has_session {
                    if let Ok(sessions) = db.borrow().get_sessions_for_habit_date(habit_id, date) {
                        if let Some(s) = sessions.first() {
                            let _ = db.borrow().delete_session(*s);
                        }
                    }
                    circle_c.remove_css_class("day-completed");
                } else {
                    let now_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    let _ = db
                        .borrow()
                        .insert_session(habit_id, date, habit_duration, &now_str);
                    circle_c.add_css_class("day-completed");
                }
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

        week_box.append(&circle);
    }

    // Edit button
    let edit_btn = gtk4::Button::with_label("⋯");
    edit_btn.add_css_class("flat");

    {
        let window = ctx.window.clone();
        let db = ctx.db.clone();
        let settings = ctx.settings.clone();
        let habits = ctx.habits.clone();
        let habit_list = ctx.habit_list.clone();
        let wsl = ctx.wsl.clone();
        let timer_shared = ctx.timer_shared.clone();
        let timer_label = ctx.timer_label.clone();
        let timer_btn = ctx.timer_btn.clone();
        let habit = habit.clone();

        edit_btn.connect_clicked(move |_| {
            super::dialogs::show_habit_menu(
                &window,
                &db,
                &settings,
                &habits,
                &habit_list,
                &wsl,
                timer_shared.clone(),
                &timer_label,
                &timer_btn,
                &habit,
            );
        });
    }

    top_box.append(&desc);
    top_box.append(&edit_btn);
    row.append(&top_box);
    row.append(&week_box);

    ctx.habit_list.append(&row);
}
