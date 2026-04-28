use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;

use crate::db::{Database, Habit};
use crate::settings::Settings;

mod clock;
pub mod dialogs;
mod habit_row;
mod timer_banner;
mod week_summary;

/// Shared mutable state for the timer, wrapped in Rc<RefCell> for GTK closures.
pub struct TimerShared {
    pub active_habit_id: RefCell<Option<i64>>,
    pub timer_source_id: RefCell<Option<glib::SourceId>>,
    pub timer_start_instant: RefCell<Option<std::time::Instant>>,
    pub timer_elapsed_before: RefCell<u32>,
    pub clock: Rc<clock::CircularClock>,
}

pub type TimerSharedRef = Rc<RefCell<TimerShared>>;

/// Bundles all shared state needed to create a habit row or interact with the timer.
pub struct RowContext<'a> {
    pub db: &'a RefCell<Database>,
    pub settings: &'a RefCell<Settings>,
    pub habits: &'a RefCell<Vec<Habit>>,
    pub habit_list: &'a gtk4::Box,
    pub wsl: &'a gtk4::Label,
    pub timer_label: &'a gtk4::Label,
    pub timer_btn: &'a gtk4::Button,
    pub timer_shared: TimerSharedRef,
    pub window: &'a gtk4::ApplicationWindow,
}

/// Helper to build a TimerBannerState from owned widgets + TimerShared refs.
pub struct BannerWidgets {
    pub timer_label: gtk4::Label,
    pub timer_btn: gtk4::Button,
}

pub struct AppView {
    root: gtk4::Box,
    pub window: gtk4::ApplicationWindow,
    pub db: RefCell<Database>,
    pub settings: RefCell<Settings>,
    pub habits: RefCell<Vec<Habit>>,
    pub timer_shared: TimerSharedRef,
    banner: BannerWidgets,
    habit_list: gtk4::Box,
    wsl: gtk4::Label,
}

impl AppView {
    pub fn new(window: &gtk4::ApplicationWindow, db: &Database, settings: &Settings) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("app-view");

        // Timer banner
        // Timer banner (vertical: clock on top, label+button below)
        let timer_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        timer_box.add_css_class("timer-banner");
        let clock = Rc::new(clock::CircularClock::new());
        clock.widget().set_hexpand(true);
        clock.widget().set_halign(gtk4::Align::Center);
        let sub_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let timer_label = gtk4::Label::new(Some("0.0 pomodoros"));
        timer_label.add_css_class("timer-banner-label");
        timer_label.set_hexpand(true);
        timer_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        timer_label.set_xalign(0.0);
        let timer_btn = gtk4::Button::with_label("Start");
        timer_btn.add_css_class("timer-banner-button");
        sub_box.append(&timer_label);
        sub_box.append(&timer_btn);
        timer_box.append(clock.widget());
        timer_box.append(&sub_box);

        // Habit list (scrollable)
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        let habit_list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        habit_list.set_margin_start(4);
        habit_list.set_margin_end(4);
        scrolled.set_child(Some(&habit_list));
        scrolled.set_vexpand(true);

        // Week summary label
        let wsl = gtk4::Label::new(None);
        wsl.set_wrap(true);

        // Add button
        let add_button = gtk4::Button::with_label("+ Add Habit");
        add_button.add_css_class("add-button");

        root.append(&timer_box);
        root.append(&scrolled);
        root.append(&wsl);
        root.append(&add_button);

        // Load CSS
        let css = include_str!("../../data/styles.css");
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(css);
        let display = gdk4::Display::default().expect("No GTK display");
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Shared timer state — Rc<RefCell> so closures can share ownership
        let timer_shared: TimerSharedRef = Rc::new(RefCell::new(TimerShared {
            active_habit_id: RefCell::new(None),
            timer_source_id: RefCell::new(None),
            timer_start_instant: RefCell::new(None),
            timer_elapsed_before: RefCell::new(0),
            clock: clock.clone(),
        }));

        // Timer button signal
        let btn_clone = timer_btn.clone();
        let timer_shared_btn = timer_shared.clone();
        let db_clone = RefCell::new(db.clone());
        let settings_clone = RefCell::new(settings.clone());
        let label_clone = timer_label.clone();

        timer_btn.connect_clicked(move |_| {
            let should_stop = {
                let s = timer_shared_btn.borrow();
                let aid = s.active_habit_id.borrow();
                aid.is_some()
            };
            if should_stop {
                timer_banner::stop(
                    &label_clone,
                    &btn_clone,
                    timer_shared_btn.clone(),
                    &db_clone,
                    &settings_clone,
                );
            }
        });

        // Timer tick — runs every 1s, updates display when timer active
        {
            let label_tick = timer_label.clone();
            let btn_tick = timer_btn.clone();
            let db_tick = RefCell::new(db.clone());
            let settings_tick = RefCell::new(settings.clone());
            glib::timeout_add_local(std::time::Duration::from_secs(1), {
                let timer_shared_t = timer_shared.clone();
                let label_t = label_tick.clone();
                let btn_t = btn_tick.clone();
                let db_t = db_tick.clone();
                let settings_t = settings_tick.clone();
                move || {
                    let aid_val: Option<i64> = {
                        let shared_b = timer_shared_t.borrow();
                        let aid_ref = shared_b.active_habit_id.borrow();
                        *aid_ref
                    };
                    if let Some(habit_id) = aid_val {
                        let habit_opt = db_t.borrow().get_habit(habit_id).ok();
                        if let Some(habit) = habit_opt {
                            timer_banner::update_tick(
                                &label_t,
                                &btn_t,
                                &habit,
                                timer_shared_t.clone(),
                                &db_t,
                                &settings_t,
                            );
                        }
                    }
                    glib::ControlFlow::Continue
                }
            });
        }

        // Add button signal
        let db_add = RefCell::new(db.clone());
        let settings_add = RefCell::new(settings.clone());
        let habits_add = RefCell::new(db.get_all_habits().unwrap_or_default());
        let habit_list_add = habit_list.clone();
        let wsl_add = wsl.clone();
        let timer_shared_add = timer_shared.clone();
        let timer_label_add = timer_label.clone();
        let timer_btn_add = timer_btn.clone();
        let window_add = window.clone();

        add_button.connect_clicked(move |_| {
            dialogs::show_add_dialog(
                &window_add,
                &db_add,
                &settings_add,
                &habits_add,
                &habit_list_add,
                &wsl_add,
                timer_shared_add.clone(),
                &timer_label_add,
                &timer_btn_add,
            );
        });

        let app = AppView {
            root: root.clone(),
            window: window.clone(),
            db: RefCell::new(db.clone()),
            settings: RefCell::new(settings.clone()),
            habits: RefCell::new(db.get_all_habits().unwrap_or_default()),
            timer_shared,
            banner: BannerWidgets {
                timer_label,
                timer_btn,
            },
            habit_list,
            wsl,
        };
        app.load_rows();
        app.update_summary();

        app
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.root
    }

    fn load_rows(&self) {
        let fresh = self.db.borrow().get_all_habits().unwrap_or_default();
        *self.habits.borrow_mut() = fresh;

        // Clear existing rows
        while let Some(child) = self.habit_list.first_child() {
            self.habit_list.remove(&child);
        }

        // Create rows
        for habit in &*self.habits.borrow() {
            let ctx = RowContext {
                db: &self.db,
                settings: &self.settings,
                habits: &self.habits,
                habit_list: &self.habit_list,
                wsl: &self.wsl,
                timer_label: &self.banner.timer_label,
                timer_btn: &self.banner.timer_btn,
                timer_shared: self.timer_shared.clone(),
                window: &self.window,
            };
            habit_row::create_row(&ctx, habit);
        }
    }

    fn update_summary(&self) {
        week_summary::update(&self.db, &self.habits, &self.wsl);
    }
}

/// Refresh from closures (called from GTK callbacks where we have RefCells but not &AppView).
pub fn refresh_from_closures(
    db: &RefCell<Database>,
    habits: &RefCell<Vec<Habit>>,
    habit_list: &gtk4::Box,
    wsl: &gtk4::Label,
    timer_shared: TimerSharedRef,
    timer_label: &gtk4::Label,
    timer_btn: &gtk4::Button,
    settings: &RefCell<Settings>,
    window: &gtk4::ApplicationWindow,
) {
    let fresh = db.borrow().get_all_habits().unwrap_or_default();
    *habits.borrow_mut() = fresh;

    // Clear and rebuild
    while let Some(child) = habit_list.first_child() {
        habit_list.remove(&child);
    }

    for habit in &*habits.borrow() {
        create_minimal_row(
            db,
            habits,
            habit_list,
            wsl,
            habit,
            timer_shared.clone(),
            timer_label,
            timer_btn,
            settings,
            window,
        );
    }
    week_summary::update(db, habits, wsl);
}

/// Create a minimal row with timer state (used from refresh_from_closures).
fn create_minimal_row(
    db: &RefCell<Database>,
    habits: &RefCell<Vec<Habit>>,
    habit_list: &gtk4::Box,
    wsl: &gtk4::Label,
    habit: &Habit,
    timer_shared: TimerSharedRef,
    timer_label: &gtk4::Label,
    timer_btn: &gtk4::Button,
    settings: &RefCell<Settings>,
    window: &gtk4::ApplicationWindow,
) {
    use chrono::Datelike;
    use pango::EllipsizeMode;

    let row = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    row.add_css_class("habit-row");

    let top_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let desc = gtk4::Label::new(Some(&habit.description));
    desc.set_ellipsize(EllipsizeMode::End);
    desc.set_hexpand(true);
    desc.set_selectable(true);
    desc.set_xalign(0.0);

    let move_up = gtk4::Button::with_label("▲");
    let move_down = gtk4::Button::with_label("▼");
    move_up.add_css_class("flat");
    move_down.add_css_class("flat");

    let week_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    week_box.set_hexpand(true);

    let today = chrono::Local::now().date_naive();
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

        let has_session = db
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
            let s = timer_shared.borrow();
            let aid = s.active_habit_id.borrow();
            *aid == Some(habit.id)
        };
        if is_today && is_active {
            circle.add_css_class("day-active");
            circle.add_css_class("day-completed");
        }

        let db_c = db.clone();
        let habits_c = habits.clone();
        let list_c = habit_list.clone();
        let wsl_c = wsl.clone();
        let timer_shared_c = timer_shared.clone();
        let timer_label_c = timer_label.clone();
        let timer_btn_c = timer_btn.clone();
        let settings_c = settings.clone();
        let window_c = window.clone();
        let habit_c = habit.clone();
        let habit_id = habit.id;
        let habit_duration = habit.timer_duration_seconds;
        let is_today = is_today;
        let has_session = has_session;
        let circle_c = circle.clone();
        let date_d = date;

        circle.connect_clicked(move |_| {
            let aid_val = {
                let s = timer_shared_c.borrow();
                let aid = s.active_habit_id.borrow();
                *aid
            };
            if is_today {
                if aid_val == Some(habit_id) {
                    timer_banner::stop(
                        &timer_label_c,
                        &timer_btn_c,
                        timer_shared_c.clone(),
                        &db_c,
                        &settings_c,
                    );
                    circle_c.remove_css_class("day-completed");
                } else {
                    // Stop old habit first if switching
                    if aid_val.is_some() {
                        timer_banner::stop(
                            &timer_label_c,
                            &timer_btn_c,
                            timer_shared_c.clone(),
                            &db_c,
                            &settings_c,
                        );
                    }
                    timer_banner::start(
                        &timer_label_c,
                        &timer_btn_c,
                        &habit_c,
                        timer_shared_c.clone(),
                        &settings_c,
                    );
                    circle_c.add_css_class("day-active");
                    circle_c.add_css_class("day-completed");
                }
            } else {
                if has_session {
                    if let Ok(sessions) =
                        db_c.borrow().get_sessions_for_habit_date(habit_id, date_d)
                    {
                        if let Some(s) = sessions.first() {
                            let _ = db_c.borrow().delete_session(*s);
                        }
                    }
                    circle_c.remove_css_class("day-completed");
                } else {
                    let now_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    let _ =
                        db_c.borrow()
                            .insert_session(habit_id, date_d, habit_duration, &now_str);
                    circle_c.add_css_class("day-completed");
                }
            }
            refresh_from_closures(
                &db_c,
                &habits_c,
                &list_c,
                &wsl_c,
                timer_shared_c.clone(),
                &timer_label_c,
                &timer_btn_c,
                &settings_c,
                &window_c,
            );
        });

        week_box.append(&circle);
    }

    {
        let db_c = db.clone();
        let habits_c = habits.clone();
        let list_c = habit_list.clone();
        let wsl_c = wsl.clone();
        let timer_shared_c = timer_shared.clone();
        let timer_label_c = timer_label.clone();
        let timer_btn_c = timer_btn.clone();
        let settings_c = settings.clone();
        let idx = habit.order_index;
        let _total = habits.borrow().len();
        let habit_id = habit.id;
        let window_c = window.clone();

        move_up.connect_clicked(move |_| {
            if idx > 0 {
                let _ = db_c.borrow_mut().move_habit(habit_id, idx - 1);
                refresh_from_closures(
                    &db_c,
                    &habits_c,
                    &list_c,
                    &wsl_c,
                    timer_shared_c.clone(),
                    &timer_label_c,
                    &timer_btn_c,
                    &settings_c,
                    &window_c,
                );
            }
        });
    }

    {
        let db_c = db.clone();
        let habits_c = habits.clone();
        let list_c = habit_list.clone();
        let wsl_c = wsl.clone();
        let timer_shared_c = timer_shared.clone();
        let timer_label_c = timer_label.clone();
        let timer_btn_c = timer_btn.clone();
        let settings_c = settings.clone();
        let idx = habit.order_index;
        let total = habits.borrow().len();
        let habit_id = habit.id;
        let window_c = window.clone();

        move_down.connect_clicked(move |_| {
            if idx < total as i32 - 1 {
                let _ = db_c.borrow_mut().move_habit(habit_id, idx + 1);
                refresh_from_closures(
                    &db_c,
                    &habits_c,
                    &list_c,
                    &wsl_c,
                    timer_shared_c.clone(),
                    &timer_label_c,
                    &timer_btn_c,
                    &settings_c,
                    &window_c,
                );
            }
        });
    }

    let edit_btn = gtk4::Button::with_label("⋯");
    edit_btn.add_css_class("flat");

    {
        let window_e = window.clone();
        let db_e = db.clone();
        let settings_e = settings.clone();
        let habits_e = habits.clone();
        let list_e = habit_list.clone();
        let wsl_e = wsl.clone();
        let timer_shared_e = timer_shared.clone();
        let timer_label_e = timer_label.clone();
        let timer_btn_e = timer_btn.clone();
        let habit_e = habit.clone();

        edit_btn.connect_clicked(move |_| {
            dialogs::show_habit_menu(
                &window_e,
                &db_e,
                &settings_e,
                &habits_e,
                &list_e,
                &wsl_e,
                timer_shared_e.clone(),
                &timer_label_e,
                &timer_btn_e,
                &habit_e,
            );
        });
    }

    top_box.append(&desc);
    top_box.append(&move_up);
    top_box.append(&move_down);
    top_box.append(&edit_btn);
    row.append(&top_box);
    row.append(&week_box);

    habit_list.append(&row);
}
