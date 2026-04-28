use chrono::{Datelike, Local};
use gtk4::prelude::*;
use pango::EllipsizeMode;

use crate::db::{Habit, HabitType};

use super::dialogs;
use super::timer_banner;
use super::RowContext;

pub fn create_row(ctx: &RowContext, habit: &Habit) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    row.add_css_class("habit-row");

    let desc = gtk4::Label::new(Some(&habit.description));
    desc.set_ellipsize(EllipsizeMode::End);
    desc.set_hexpand(true);
    desc.set_xalign(0.0);
    desc.add_css_class("habit-desc");

    let week_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    week_box.set_valign(gtk4::Align::Center);

    let today = Local::now().date_naive();
    let today_dow = today.weekday().number_from_monday();
    let week_start = today - chrono::Duration::days((today_dow as u64 - 1) as i64);

    for i in 0i64..7 {
        let date = week_start + chrono::Duration::days(i);
        let is_today = i == (today_dow as i64 - 1);

        // Compute state
        let (circle_label, is_completed, is_partial, has_data) = match habit.habit_type {
            HabitType::Timer => {
                let has = ctx.db().borrow().has_session_for_date(habit.id, date).unwrap_or(false);
                let mins = ctx.db().borrow().get_total_minutes_for_habit_date(habit.id, date).unwrap_or(0);
                let label = if has { format!("{}m", mins) } else { day_letter(date.weekday().number_from_monday() as i64) };
                (label, has, false, has)
            }
            HabitType::Boolean => {
                let val = ctx.db().borrow().get_value_for_habit_date(habit.id, date).unwrap_or(None);
                let yes = val.is_some();
                ("✓".into(), yes, false, true)
            }
            HabitType::Number => {
                let val = ctx.db().borrow().get_value_for_habit_date(habit.id, date).unwrap_or(None);
                match (val, habit.min_value) {
                    (Some(v), Some(min)) => {
                        if (v as u32) >= min {
                            (v.to_string(), true, false, true)
                        } else {
                            (v.to_string(), false, true, true)
                        }
                    }
                    (Some(v), None) => (v.to_string(), true, false, true),
                    (None, _) => (day_letter(date.weekday().number_from_monday() as i64), false, false, false),
                }
            }
        };

        let circle = gtk4::Button::with_label(&circle_label);
        circle.add_css_class("flat");
        circle.add_css_class("day-circle");
        circle.set_size_request(30, 30);
        circle.set_halign(gtk4::Align::Center);
        circle.set_valign(gtk4::Align::Center);

        if is_completed {
            circle.add_css_class("day-completed");
        }
        if is_partial {
            circle.add_css_class("day-partial");
        }
        if is_today {
            circle.add_css_class("day-today");
        }

        let is_active = {
            let s = ctx.timer_shared.borrow();
            let aid = s.active_habit_id.borrow();
            *aid == Some(habit.id)
        };
        if is_active && habit.habit_type == HabitType::Timer {
            row.add_css_class("active");
        }
        if is_today && is_active && habit.habit_type == HabitType::Timer {
            circle.add_css_class("day-active");
            circle.add_css_class("day-completed");
        }

        // === Left-click handler ===
        {
            let db = ctx.db().clone();
            let habits = ctx.habits().clone();
            let habit_list = ctx.habit_list.clone();
            let wsl = ctx.wsl.clone();
            let timer_label = ctx.timer_label.clone();
            let timer_btn = ctx.timer_btn.clone();
            let timer_shared = ctx.timer_shared.clone();
            let settings = ctx.settings.clone();
            let window = ctx.window.clone();
            let habit = habit.clone();
            let is_today = is_today;
            let circle_c = circle.clone();
            let row_c = row.clone();

            circle.connect_clicked(move |_| {
                if habit.habit_type != HabitType::Timer {
                    non_timer_click(&habit, &db, &settings, &habits, &habit_list, &wsl,
                        timer_shared.clone(), &timer_label, &timer_btn, &window,
                        circle_c.clone(), row_c.clone(), date);
                    return;
                }

                let aid_val = {
                    let s = timer_shared.borrow();
                    let aid = s.active_habit_id.borrow();
                    *aid
                };
                if is_today {
                    if aid_val == Some(habit.id) {
                        timer_banner::stop(&timer_label, &timer_btn, timer_shared.clone(), &db, &settings);
                        circle_c.remove_css_class("day-completed");
                        circle_c.remove_css_class("day-active");
                        row_c.remove_css_class("active");
                        let ts = timer_shared.borrow();
                        ts.active_row.set(None);
                        ts.active_circle.set(None);
                        ts.active_habit_for_circle.replace(None);
                        // Update circle label to show saved minutes
                        let mins = db.borrow().get_total_minutes_for_habit_date(habit.id, date).unwrap_or(0);
                        if mins > 0 {
                            circle_c.set_label(&format!("{}m", mins));
                            circle_c.add_css_class("day-completed");
                        } else {
                            circle_c.set_label(&day_letter(date.weekday().number_from_monday() as i64));
                        }
                    } else {
                        if aid_val.is_some() {
                            timer_banner::stop(&timer_label, &timer_btn, timer_shared.clone(), &db, &settings);
                        }
                        let old_row = {
                            let ts = timer_shared.borrow();
                            ts.active_row.replace(None)
                        };
                        let old_circle = {
                            let ts = timer_shared.borrow();
                            ts.active_circle.replace(None)
                        };
                        let old_habit_id = {
                            let ts = timer_shared.borrow();
                            ts.active_habit_for_circle.replace(None)
                        };
                        if let Some(or) = old_row {
                            or.remove_css_class("active");
                        }
                        // Update old circle label to show saved minutes
                        if let Some(oc) = old_circle {
                            oc.remove_css_class("day-active");
                            if let Some(old_id) = old_habit_id {
                                let today = Local::now().date_naive();
                                let mins = db.borrow().get_total_minutes_for_habit_date(old_id, today).unwrap_or(0);
                                if mins > 0 {
                                    oc.set_label(&format!("{}m", mins));
                                    oc.add_css_class("day-completed");
                                } else {
                                    oc.set_label(&day_letter(today.weekday().number_from_monday() as i64));
                                }
                            }
                        }
                        let mode = timer_shared.borrow().clock.get_mode();
                        timer_banner::start(&timer_label, &timer_btn, &habit, mode, timer_shared.clone(), &settings);
                        circle_c.add_css_class("day-active");
                        circle_c.add_css_class("day-completed");
                        row_c.add_css_class("active");
                        let ts = timer_shared.borrow();
                        ts.active_row.replace(Some(row_c.clone()));
                        ts.active_circle.replace(Some(circle_c.clone()));
                        ts.active_habit_for_circle.replace(Some(habit.id));
                    }
                } else {
                    let has_session = db.borrow().has_session_for_date(habit.id, date).unwrap_or(false);
                    if has_session {
                        dialogs::show_edit_session_time(&window, &db, &settings, &habits, &habit_list, &wsl,
                            timer_shared.clone(), &timer_label, &timer_btn, habit.id, date);
                    } else {
                        let now_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        let _ = db.borrow().insert_session(habit.id, date, habit.timer_duration_seconds, &now_str);
                        let mins = db.borrow().get_total_minutes_for_habit_date(habit.id, date).unwrap_or(0);
                        circle_c.set_label(&format!("{}m", mins));
                        circle_c.add_css_class("day-completed");
                    }
                }
            });
        }

        // === Right-click context menu ===
        if habit.habit_type == HabitType::Timer || has_data {
            let popover = gtk4::Popover::new();
            popover.set_position(gtk4::PositionType::Bottom);
            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            content.set_margin_start(8);
            content.set_margin_end(8);
            content.set_margin_top(8);
            content.set_margin_bottom(8);

            match habit.habit_type {
                HabitType::Timer => {
                    let btn_add = gtk4::Button::with_label("Add time");
                    let btn_edit = gtk4::Button::with_label("Edit time");
                    let btn_remove = gtk4::Button::with_label("Remove time");
                    content.append(&btn_add);
                    content.append(&btn_edit);
                    content.append(&btn_remove);
                    if !has_data {
                        btn_add.set_visible(true);
                        btn_edit.set_visible(false);
                        btn_remove.set_visible(false);
                    }
                    let habit_id = habit.id;
                    let date_c = date;
                    {
                        let db = ctx.db().clone();
                        let habits = ctx.habits().clone();
                        let habit_list = ctx.habit_list.clone();
                        let wsl = ctx.wsl.clone();
                        let timer_shared = ctx.timer_shared.clone();
                        let timer_label = ctx.timer_label.clone();
                        let timer_btn = ctx.timer_btn.clone();
                        let settings = ctx.settings.clone();
                        let window = ctx.window.clone();
                        let popover_c = popover.clone();
                        btn_add.connect_clicked(move |_| {
                            popover_c.popdown();
                            dialogs::show_edit_session_time(&window, &db, &settings, &habits, &habit_list, &wsl,
                                timer_shared.clone(), &timer_label, &timer_btn, habit_id, date_c);
                        });
                    }
                    {
                        let db = ctx.db().clone();
                        let habits = ctx.habits().clone();
                        let habit_list = ctx.habit_list.clone();
                        let wsl = ctx.wsl.clone();
                        let timer_shared = ctx.timer_shared.clone();
                        let timer_label = ctx.timer_label.clone();
                        let timer_btn = ctx.timer_btn.clone();
                        let settings = ctx.settings.clone();
                        let window = ctx.window.clone();
                        let popover_c = popover.clone();
                        btn_edit.connect_clicked(move |_| {
                            popover_c.popdown();
                            dialogs::show_edit_session_time(&window, &db, &settings, &habits, &habit_list, &wsl,
                                timer_shared.clone(), &timer_label, &timer_btn, habit_id, date_c);
                        });
                    }
                    {
                        let db = ctx.db().clone();
                        let habits = ctx.habits().clone();
                        let habit_list = ctx.habit_list.clone();
                        let wsl = ctx.wsl.clone();
                        let timer_shared = ctx.timer_shared.clone();
                        let timer_label = ctx.timer_label.clone();
                        let timer_btn = ctx.timer_btn.clone();
                        let settings = ctx.settings.clone();
                        let window = ctx.window.clone();
                        let circle_c = circle.clone();
                        let popover_c = popover.clone();
                        btn_remove.connect_clicked(move |_| {
                            popover_c.popdown();
                            let _ = db.borrow().delete_sessions_for_habit_date(habit_id, date_c);
                            circle_c.remove_css_class("day-completed");
                            super::refresh_from_closures_compat(&db, &habits, &habit_list, &wsl, timer_shared.clone(), &timer_label, &timer_btn, &settings, &window);
                        });
                    }
                }
                HabitType::Boolean => {
                    let btn_set_yes = gtk4::Button::with_label("Set Yes");
                    let btn_set_no = gtk4::Button::with_label("Set No");
                    content.append(&btn_set_yes);
                    content.append(&btn_set_no);
                    let habit_id = habit.id;
                    let date_c = date;
                    {
                        let db = ctx.db().clone();
                        let circle_c = circle.clone();
                        let popover_c = popover.clone();
                        btn_set_yes.connect_clicked(move |_| {
                            popover_c.popdown();
                            let _ = db.borrow().set_value_for_habit_date(habit_id, date_c, 1.0);
                            circle_c.add_css_class("day-completed");
                        });
                    }
                    {
                        let db = ctx.db().clone();
                        let circle_c = circle.clone();
                        let popover_c = popover.clone();
                        btn_set_no.connect_clicked(move |_| {
                            popover_c.popdown();
                            let _ = db.borrow().clear_value_for_habit_date(habit_id, date_c);
                            circle_c.remove_css_class("day-completed");
                        });
                    }
                }
                HabitType::Number => {
                    let btn_edit = gtk4::Button::with_label("Edit value");
                    let btn_remove = gtk4::Button::with_label("Remove value");
                    content.append(&btn_edit);
                    content.append(&btn_remove);
                    let habit_id = habit.id;
                    let date_c = date;
                    {
                        let db = ctx.db().clone();
                        let habits = ctx.habits().clone();
                        let habit_list = ctx.habit_list.clone();
                        let wsl = ctx.wsl.clone();
                        let timer_shared = ctx.timer_shared.clone();
                        let timer_label = ctx.timer_label.clone();
                        let timer_btn = ctx.timer_btn.clone();
                        let settings = ctx.settings.clone();
                        let window = ctx.window.clone();
                        let popover_c = popover.clone();
                        let min_value = habit.min_value;
                        btn_edit.connect_clicked(move |_| {
                            popover_c.popdown();
                            dialogs::show_edit_number_value(&window, &db, &settings, &habits, &habit_list, &wsl,
                                timer_shared.clone(), &timer_label, &timer_btn, habit_id, date_c, min_value);
                        });
                    }
                    {
                        let db = ctx.db().clone();
                        let habits = ctx.habits().clone();
                        let habit_list = ctx.habit_list.clone();
                        let wsl = ctx.wsl.clone();
                        let timer_shared = ctx.timer_shared.clone();
                        let timer_label = ctx.timer_label.clone();
                        let timer_btn = ctx.timer_btn.clone();
                        let settings = ctx.settings.clone();
                        let window = ctx.window.clone();
                        let circle_c = circle.clone();
                        let popover_c = popover.clone();
                        btn_remove.connect_clicked(move |_| {
                            popover_c.popdown();
                            let _ = db.borrow().clear_value_for_habit_date(habit_id, date_c);
                            circle_c.remove_css_class("day-completed");
                            circle_c.remove_css_class("day-partial");
                            super::refresh_from_closures_compat(&db, &habits, &habit_list, &wsl, timer_shared.clone(), &timer_label, &timer_btn, &settings, &window);
                        });
                    }
                }
            }

            popover.set_child(Some(&content));
            let circle_wrapper = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            circle_wrapper.append(&circle);
            circle_wrapper.append(&popover);

            let gesture = gtk4::GestureClick::builder()
                .button(gtk4::gdk::BUTTON_SECONDARY)
                .build();
            let popover_c = popover.clone();
            gesture.connect_released(move |_g, _x, _y, _| {
                popover_c.popup();
            });
            circle_wrapper.add_controller(gesture);
            week_box.append(&circle_wrapper);
        } else {
            week_box.append(&circle);
        }
    }

    build_context_menu(ctx, &row, habit);

    row.append(&desc);
    row.append(&week_box);
    ctx.habit_list.append(&row);
}

fn day_letter(dow: i64) -> String {
    match dow {
        1 => "M".into(), 2 => "T".into(), 3 => "W".into(), 4 => "T".into(),
        5 => "F".into(), 6 => "S".into(), 7 => "S".into(), _ => "?".into(),
    }
}

fn non_timer_click(
    habit: &Habit, db: &std::cell::RefCell<crate::db::Database>, settings: &std::cell::RefCell<crate::settings::Settings>,
    habits: &std::cell::RefCell<Vec<Habit>>, habit_list: &gtk4::Box, wsl: &gtk4::Label,
    timer_shared: std::rc::Rc<std::cell::RefCell<super::TimerShared>>, timer_label: &gtk4::Label,
    timer_btn: &gtk4::Button, window: &gtk4::ApplicationWindow, circle: gtk4::Button, _row: gtk4::Box, date: chrono::NaiveDate,
) {
    match habit.habit_type {
        HabitType::Boolean => {
            let current = db.borrow().get_value_for_habit_date(habit.id, date).unwrap_or(None);
            if current.is_some() {
                let _ = db.borrow().clear_value_for_habit_date(habit.id, date);
                circle.remove_css_class("day-completed");
            } else {
                let _ = db.borrow().set_value_for_habit_date(habit.id, date, 1.0);
                circle.add_css_class("day-completed");
            }
        }
        HabitType::Number => {
            dialogs::show_edit_number_value(window, db, settings, habits, habit_list, wsl,
                timer_shared.clone(), timer_label, timer_btn, habit.id, date, habit.min_value);
        }
        HabitType::Timer => {}
    }
}

fn build_context_menu(ctx: &RowContext, row: &gtk4::Box, habit: &Habit) {
    let popover = gtk4::Popover::new();
    popover.set_position(gtk4::PositionType::Bottom);
    let popover_content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    popover_content.set_margin_start(8);
    popover_content.set_margin_end(8);
    popover_content.set_margin_top(8);
    popover_content.set_margin_bottom(8);

    let menu_edit = gtk4::Button::with_label("Edit");
    let menu_delete = gtk4::Button::with_label("Delete");
    popover_content.append(&menu_edit);
    popover_content.append(&menu_delete);

    if habit.habit_type == HabitType::Timer {
        let menu_start = gtk4::Button::with_label("Start Pomodoro");
        popover_content.append(&menu_start);
        {
            let timer_shared = ctx.timer_shared.clone();
            let timer_label = ctx.timer_label.clone();
            let timer_btn = ctx.timer_btn.clone();
            let db = ctx.db().clone();
            let settings = ctx.settings.clone();
            let habit = habit.clone();
            menu_start.connect_clicked(move |_| {
                let aid_val = { let s = timer_shared.borrow(); let aid = s.active_habit_id.borrow(); *aid };
                if aid_val.is_some() {
                    timer_banner::stop(&timer_label, &timer_btn, timer_shared.clone(), &db, &settings);
                }
                let mode = timer_shared.borrow().clock.get_mode();
                timer_banner::start(&timer_label, &timer_btn, &habit, mode, timer_shared.clone(), &settings);
            });
        }
    }

    popover.set_child(Some(&popover_content));

    {
        let window = ctx.window.clone(); let db = ctx.db().clone(); let settings = ctx.settings.clone();
        let habits = ctx.habits().clone(); let habit_list = ctx.habit_list.clone(); let wsl = ctx.wsl.clone();
        let timer_shared = ctx.timer_shared.clone(); let timer_label = ctx.timer_label.clone();
        let timer_btn = ctx.timer_btn.clone(); let habit = habit.clone();
        menu_edit.connect_clicked(move |_| {
            dialogs::show_edit_dialog(&window, &db, &settings, &habits, &habit_list, &wsl,
                timer_shared.clone(), &timer_label, &timer_btn, &habit);
        });
    }

    {
        let window = ctx.window.clone(); let db = ctx.db().clone(); let settings = ctx.settings.clone();
        let habits = ctx.habits().clone(); let habit_list = ctx.habit_list.clone(); let wsl = ctx.wsl.clone();
        let timer_shared = ctx.timer_shared.clone(); let timer_label = ctx.timer_label.clone();
        let timer_btn = ctx.timer_btn.clone(); let habit = habit.clone();
        menu_delete.connect_clicked(move |_| {
            dialogs::delete_habit(&window, &db, &settings, &habits, &habit_list, &wsl,
                timer_shared.clone(), &timer_label, &timer_btn, &habit);
        });
    }

    row.append(&popover);

    let gesture = gtk4::GestureClick::builder().button(gtk4::gdk::BUTTON_SECONDARY).build();
    let popover_c = popover.clone();
    gesture.connect_released(move |_g, _x, _y, _| { popover_c.popup(); });
    row.add_controller(gesture);
}

