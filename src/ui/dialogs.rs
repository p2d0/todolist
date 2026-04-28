use std::cell::RefCell;

use gtk4::prelude::*;

use crate::db::{Database, Habit, HabitMode};
use crate::settings::Settings;

type TimerSharedRef = std::rc::Rc<RefCell<crate::ui::TimerShared>>;

/// Show the "Add Habit" dialog.
pub fn show_add_dialog(
    window: &gtk4::ApplicationWindow,
    db: &RefCell<Database>,
    settings: &RefCell<Settings>,
    habits: &RefCell<Vec<Habit>>,
    habit_list: &gtk4::Box,
    wsl: &gtk4::Label,
    timer_shared: TimerSharedRef,
    timer_label: &gtk4::Label,
    timer_btn: &gtk4::Button,
) {
    let dialog = gtk4::Dialog::new();
    dialog.set_title(Some("Add Habit"));
    dialog.set_transient_for(Some(window));
    dialog.set_destroy_with_parent(true);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    let desc_entry = gtk4::Entry::new();
    desc_entry.set_placeholder_text(Some("Habit description..."));

    let mode_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let mode_timed = gtk4::CheckButton::with_label("Timed (countdown)");
    let mode_stopwatch = gtk4::CheckButton::with_label("Stopwatch");
    mode_stopwatch.set_active(true);
    mode_box.append(&mode_timed);
    mode_box.append(&mode_stopwatch);

    let duration_spin = gtk4::SpinButton::with_range(1.0, 600.0, 1.0);
    duration_spin.set_value(25.0);
    duration_spin.set_digits(0);
    let duration_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let dur_label = gtk4::Label::new(Some("Duration (min):"));
    duration_box.append(&dur_label);
    duration_box.append(&duration_spin);

    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let add_btn = gtk4::Button::with_label("Add");
    add_btn.add_css_class("suggested-action");
    btn_box.append(&cancel_btn);
    btn_box.append(&add_btn);
    add_btn.set_hexpand(true);
    add_btn.set_halign(gtk4::Align::End);

    content.append(&desc_entry);
    content.append(&mode_box);
    content.append(&duration_box);
    content.append(&btn_box);

    dialog.set_child(Some(&content));

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog.close();
        });
    }

    {
        let db = RefCell::new(db.borrow().clone());
        let habits = RefCell::new(habits.borrow().clone());
        let habit_list = habit_list.clone();
        let wsl = wsl.clone();
        let dialog = dialog.clone();
        let timer_shared = timer_shared.clone();
        let timer_label = timer_label.clone();
        let timer_btn = timer_btn.clone();
        let settings = RefCell::new(settings.borrow().clone());
        let window = window.clone();

        add_btn.connect_clicked(move |_| {
            let desc = desc_entry.text().to_string();
            if desc.trim().is_empty() {
                return;
            }
            let mode = if mode_timed.is_active() {
                HabitMode::Timed
            } else {
                HabitMode::Stopwatch
            };
            let duration = duration_spin.value() as u32 * 60;

            let _ = db.borrow_mut().add_habit(&desc, duration, mode);
            dialog.close();
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

    dialog.present();
}

/// Show edit dialog for a habit.
pub fn show_edit_dialog(
    window: &gtk4::ApplicationWindow,
    db: &RefCell<Database>,
    settings: &RefCell<Settings>,
    habits: &RefCell<Vec<Habit>>,
    habit_list: &gtk4::Box,
    wsl: &gtk4::Label,
    timer_shared: TimerSharedRef,
    timer_label: &gtk4::Label,
    timer_btn: &gtk4::Button,
    habit: &Habit,
) {
    let dialog = gtk4::Dialog::new();
    dialog.set_title(Some("Edit Habit"));
    dialog.set_transient_for(Some(window));
    dialog.set_destroy_with_parent(true);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    let desc_entry = gtk4::Entry::new();
    desc_entry.set_text(&habit.description);

    let mode_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let mode_timed = gtk4::CheckButton::with_label("Timed");
    let mode_stopwatch = gtk4::CheckButton::with_label("Stopwatch");
    if habit.mode == HabitMode::Timed {
        mode_timed.set_active(true);
    } else {
        mode_stopwatch.set_active(true);
    }
    mode_box.append(&mode_timed);
    mode_box.append(&mode_stopwatch);

    let duration_min = habit.timer_duration_seconds.div_ceil(60);
    let duration_spin = gtk4::SpinButton::with_range(0.0, 600.0, 1.0);
    duration_spin.set_value(duration_min as f64);
    duration_spin.set_digits(0);
    let duration_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let dur_label = gtk4::Label::new(Some("Duration (min):"));
    duration_box.append(&dur_label);
    duration_box.append(&duration_spin);

    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);

    content.append(&desc_entry);
    content.append(&mode_box);
    content.append(&duration_box);
    content.append(&btn_box);
    dialog.set_child(Some(&content));

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog.close();
        });
    }

    {
        let db = RefCell::new(db.borrow().clone());
        let habits = RefCell::new(habits.borrow().clone());
        let habit_list = habit_list.clone();
        let wsl = wsl.clone();
        let dialog = dialog.clone();
        let habit_id = habit.id;
        let timer_shared = timer_shared.clone();
        let timer_label = timer_label.clone();
        let timer_btn = timer_btn.clone();
        let settings = RefCell::new(settings.borrow().clone());
        let window = window.clone();

        save_btn.connect_clicked(move |_| {
            let desc = desc_entry.text().to_string();
            if desc.trim().is_empty() {
                return;
            }
            let mode = if mode_timed.is_active() {
                HabitMode::Timed
            } else {
                HabitMode::Stopwatch
            };
            let duration = duration_spin.value() as u32 * 60;

            let _ = db
                .borrow_mut()
                .update_habit(habit_id, &desc, duration, mode);
            dialog.close();
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

    dialog.present();
}

/// Show a popup menu for habit actions (edit, delete).
pub fn show_habit_menu(
    window: &gtk4::ApplicationWindow,
    db: &RefCell<Database>,
    settings: &RefCell<Settings>,
    habits: &RefCell<Vec<Habit>>,
    habit_list: &gtk4::Box,
    wsl: &gtk4::Label,
    timer_shared: TimerSharedRef,
    timer_label: &gtk4::Label,
    timer_btn: &gtk4::Button,
    habit: &Habit,
) {
    let dialog = gtk4::Dialog::new();
    dialog.set_title(Some("Actions"));
    dialog.set_transient_for(Some(window));
    dialog.set_destroy_with_parent(true);
    dialog.set_default_size(150, -1);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let edit_btn = gtk4::Button::with_label("✏️ Edit");
    let delete_btn = gtk4::Button::with_label("🗑️ Delete");
    let cancel_btn = gtk4::Button::with_label("Cancel");

    content.append(&edit_btn);
    content.append(&delete_btn);
    content.append(&cancel_btn);
    dialog.set_child(Some(&content));

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog.close();
        });
    }

    {
        let db_edit = RefCell::new(db.borrow().clone());
        let habits_edit = RefCell::new(habits.borrow().clone());
        let habit_list_edit = habit_list.clone();
        let wsl_edit = wsl.clone();
        let window_edit = window.clone();
        let settings_edit = RefCell::new(settings.borrow().clone());
        let timer_shared_edit = timer_shared.clone();
        let timer_label_edit = timer_label.clone();
        let timer_btn_edit = timer_btn.clone();
        let habit_edit = habit.clone();
        let dialog_edit = dialog.clone();

        let db_del = RefCell::new(db.borrow().clone());
        let habits_del = RefCell::new(habits.borrow().clone());
        let habit_list_del = habit_list.clone();
        let wsl_del = wsl.clone();
        let window_del = window.clone();
        let timer_shared_del = timer_shared.clone();
        let timer_label_del = timer_label.clone();
        let timer_btn_del = timer_btn.clone();
        let habit_del = habit.clone();
        let dialog_del = dialog.clone();
        let settings_del = RefCell::new(settings.borrow().clone());

        edit_btn.connect_clicked(move |_| {
            dialog_edit.close();
            show_edit_dialog(
                &window_edit,
                &db_edit,
                &settings_edit,
                &habits_edit,
                &habit_list_edit,
                &wsl_edit,
                timer_shared_edit.clone(),
                &timer_label_edit,
                &timer_btn_edit,
                &habit_edit,
            );
        });

        delete_btn.connect_clicked(move |_| {
            let id = habit_del.id;
            let desc = habit_del.description.clone();
            dialog_del.close();

            let confirm = gtk4::Dialog::new();
            confirm.set_title(Some("Delete Habit"));
            confirm.set_transient_for(Some(&window_del));
            confirm.set_destroy_with_parent(true);

            let msg = gtk4::Label::new(Some(&format!("Delete \"{}\"?", desc)));
            msg.set_wrap(true);
            let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            let yes_btn = gtk4::Button::with_label("Yes");
            yes_btn.add_css_class("destructive-action");
            let no_btn = gtk4::Button::with_label("No");
            btn_box.append(&no_btn);
            btn_box.append(&yes_btn);

            let inner = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
            inner.set_margin_start(12);
            inner.set_margin_end(12);
            inner.set_margin_top(12);
            inner.set_margin_bottom(12);
            inner.append(&msg);
            inner.append(&btn_box);
            confirm.set_child(Some(&inner));

            let confirm_no = confirm.clone();
            no_btn.connect_clicked(move |_| {
                confirm_no.close();
            });

            {
                let db = RefCell::new(db_del.borrow().clone());
                let habits = RefCell::new(habits_del.borrow().clone());
                let habit_list = habit_list_del.clone();
                let wsl = wsl_del.clone();
                let timer_shared = timer_shared_del.clone();
                let timer_label = timer_label_del.clone();
                let timer_btn = timer_btn_del.clone();
                let settings = settings_del.clone();
                let window = window_del.clone();

                let confirm_yes = confirm.clone();
                yes_btn.connect_clicked(move |_| {
                    let _ = db.borrow_mut().delete_habit(id);
                    let borrowed = timer_shared.borrow();
                    let aid = borrowed.active_habit_id.borrow();
                    if *aid == Some(id) {
                        borrowed.active_habit_id.replace(None);
                    }
                    confirm_yes.close();
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

            confirm.present();
        });
    }

    dialog.present();
}
