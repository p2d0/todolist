use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;

use crate::db::{Database, Habit, HabitMode};
use crate::service::AppState;
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
    pub active_row: Rc<Cell<Option<gtk4::Box>>>,
    pub active_circle: Rc<Cell<Option<gtk4::Button>>>,
    pub active_habit_for_circle: RefCell<Option<i64>>,
}

pub type TimerSharedRef = Rc<RefCell<TimerShared>>;

/// Bundles all shared state needed to create a habit row or interact with the timer.
pub struct RowContext<'a> {
    pub app_state: Rc<AppState>,
    pub settings: &'a RefCell<Settings>,
    pub habit_list: &'a gtk4::Box,
    pub wsl: &'a gtk4::Label,
    pub timer_label: &'a gtk4::Label,
    pub timer_btn: &'a gtk4::Button,
    pub timer_shared: TimerSharedRef,
    pub window: &'a gtk4::ApplicationWindow,
}

// Convenience getters for backward compat in closures
impl<'a> RowContext<'a> {
    pub fn db(&self) -> &RefCell<Database> {
        &self.app_state.db
    }
    pub fn habits(&self) -> &RefCell<Vec<Habit>> {
        &self.app_state.habits
    }
}

/// Helper to build a TimerBannerState from owned widgets + TimerShared refs.
pub struct BannerWidgets {
    pub timer_label: gtk4::Label,
    pub timer_btn: gtk4::Button,
    pub _timer_mode_btn: gtk4::Button,
}

pub struct AppView {
    root: gtk4::Box,
    pub window: gtk4::ApplicationWindow,
    pub app_state: Rc<AppState>,
    pub settings: RefCell<Settings>,
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
        // Mode toggle button (P = Pomodoro, S = Stopwatch)
        let timer_mode_btn = gtk4::Button::with_label("P");
        timer_mode_btn.add_css_class("timer-mode-button");
        sub_box.append(&timer_label);
        sub_box.append(&timer_mode_btn);
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
        add_button.set_hexpand(true);

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
            active_row: Rc::new(Cell::new(None)),
            active_circle: Rc::new(Cell::new(None)),
            active_habit_for_circle: RefCell::new(None),
        }));

        // Mode toggle button — click to switch between Pomodoro and Stopwatch
        {
            let clock_m = clock.clone();
            timer_mode_btn.connect_clicked(move |btn| {
                let current = clock_m.get_mode();
                let next = if matches!(current, HabitMode::Timed) {
                    HabitMode::Stopwatch
                } else {
                    HabitMode::Timed
                };
                clock_m.set_mode(next);
                btn.set_label(if matches!(next, HabitMode::Timed) { "P" } else { "S" });
            });
        }

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
                // Update the active circle label to show saved minutes
                let (circle, habit_id, old_row) = {
                    let ts = timer_shared_btn.borrow();
                    let c = ts.active_circle.replace(None);
                    let h = ts.active_habit_for_circle.replace(None);
                    let r = ts.active_row.replace(None);
                    (c, h, r)
                };
                if let Some(cir) = circle {
                    if let Some(hid) = habit_id {
                        let today = chrono::Local::now().naive_local().date();
                        let mins = db_clone.borrow().get_total_minutes_for_habit_date(hid, today).unwrap_or(0);
                        let label_text = if mins > 0 { format!("{}m", mins) } else { "?".to_string() };
                        cir.set_label(&label_text);
                        if mins > 0 {
                            cir.add_css_class("day-completed");
                        }
                        cir.remove_css_class("day-active");
                    }
                }
                if let Some(r) = old_row {
                    r.remove_css_class("active");
                }
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
            app_state: Rc::new(AppState::new(db)),
            settings: RefCell::new(settings.clone()),
            timer_shared,
            banner: BannerWidgets {
                timer_label,
                timer_btn,
                _timer_mode_btn: timer_mode_btn,
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
        self.app_state.reload_habits();

        // Clear existing rows
        while let Some(child) = self.habit_list.first_child() {
            self.habit_list.remove(&child);
        }

        // Create rows
        for habit in &*self.app_state.habits.borrow() {
            let ctx = RowContext {
                            app_state: self.app_state.clone(),
                            settings: &self.settings,
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
        week_summary::update(&self.app_state.db, &self.app_state.habits, &self.wsl);
    }
}

/// Refresh from closures (called from GTK callbacks where we have Rc<AppState> + other widgets).
    #[allow(dead_code)]
pub fn refresh_from_closures(
    app_state: &Rc<AppState>,
    habit_list: &gtk4::Box,
    wsl: &gtk4::Label,
    timer_shared: TimerSharedRef,
    timer_label: &gtk4::Label,
    timer_btn: &gtk4::Button,
    settings: &RefCell<Settings>,
    window: &gtk4::ApplicationWindow,
) {
    app_state.reload_habits();
    _refresh_from_closures_impl(&app_state.db, &app_state.habits, habit_list, wsl, timer_shared, timer_label, timer_btn, settings, window);
}

/// Backward compat: refresh with separate db and habits (used by dialogs/popover).
pub fn refresh_from_closures_compat(
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
    _refresh_from_closures_impl(db, habits, habit_list, wsl, timer_shared, timer_label, timer_btn, settings, window);
}

fn _refresh_from_closures_impl(
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
        // Rebuild rows — closures use old-style ctx but that's fine for now
        // TODO: migrate to app_state-based ctx
        let ctx = RowContext {
            app_state: Rc::new(AppState {
                db: db.clone(),
                habits: habits.clone(),
            }),
            settings,
            habit_list,
            wsl,
            timer_label,
            timer_btn,
            timer_shared: timer_shared.clone(),
            window,
        };
        habit_row::create_row(&ctx, habit);
    }
    week_summary::update(db, habits, wsl);
}

