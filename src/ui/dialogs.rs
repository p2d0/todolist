use std::cell::RefCell;

use gtk4::prelude::*;

use crate::db::{Database, Habit, HabitMode, HabitType};
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

    // Habit type selector
    let type_label = gtk4::Label::new(Some("Type:"));
    type_label.set_xalign(0.0);
    let type_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    type_box.append(&type_label);
    let type_view = gtk4::ComboBoxText::new();
    type_view.append_text("Timer");
    type_view.append_text("Yes/No");
    type_view.append_text("Number");
    type_view.set_active(Some(0)); // Timer by default
    type_box.append(&type_view);

    // Min value (only shown for Number type)
    let min_spin = gtk4::SpinButton::with_range(0.0, 10000.0, 1.0);
    min_spin.set_value(0.0);
    min_spin.set_digits(0);
    let min_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let min_label = gtk4::Label::new(Some("Min (optional):"));
    min_box.append(&min_label);
    min_box.append(&min_spin);
    min_box.set_visible(false);

    // Show/hide min_box based on type selection
    {
        let min_box = min_box.clone();
        type_view.connect_changed(move |combo| {
            let idx = combo.active();
            min_box.set_visible(idx == Some(2)); // 2 = Number
        });
    }

    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let add_btn = gtk4::Button::with_label("Add");
    add_btn.add_css_class("suggested-action");
    btn_box.append(&cancel_btn);
    btn_box.append(&add_btn);
    add_btn.set_hexpand(true);
    add_btn.set_halign(gtk4::Align::End);

    content.append(&desc_entry);
    content.append(&type_box);
    content.append(&min_box);
    content.append(&btn_box);

    dialog.set_child(Some(&content));

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog.close();
        });
    }

    {
        let db = db.clone();
        let habits = habits.clone();
        let habit_list = habit_list.clone();
        let wsl = wsl.clone();
        let dialog = dialog.clone();
        let timer_shared = timer_shared.clone();
        let timer_label = timer_label.clone();
        let timer_btn = timer_btn.clone();
        let settings = settings.clone();
        let window = window.clone();

        add_btn.connect_clicked(move |_| {
            let desc = desc_entry.text().to_string();
            if desc.trim().is_empty() {
                return;
            }
            let habit_type = match type_view.active() {
                Some(1) => HabitType::Boolean,
                Some(2) => HabitType::Number,
                _ => HabitType::Timer,
            };
            let min_value = if habit_type == HabitType::Number {
                let mv = min_spin.value() as u32;
                if mv > 0 { Some(mv) } else { None }
            } else {
                None
            };

            let _ = db.borrow_mut().add_habit(&desc, 1500, HabitMode::Stopwatch, habit_type, min_value);
            dialog.close();
            super::refresh_from_closures_compat(
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

    // Habit type selector
    let type_label = gtk4::Label::new(Some("Type:"));
    type_label.set_xalign(0.0);
    let type_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    type_box.append(&type_label);
    let type_view = gtk4::ComboBoxText::new();
    type_view.append_text("Timer");
    type_view.append_text("Yes/No");
    type_view.append_text("Number");
    let type_idx = match habit.habit_type {
        HabitType::Boolean => 1,
        HabitType::Number => 2,
        _ => 0,
    };
    type_view.set_active(Some(type_idx as u32));

    // Min value (only shown for Number type)
    let min_spin = gtk4::SpinButton::with_range(0.0, 10000.0, 1.0);
    min_spin.set_value(habit.min_value.unwrap_or(0) as f64);
    min_spin.set_digits(0);
    let min_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let min_label = gtk4::Label::new(Some("Min (optional):"));
    min_box.append(&min_label);
    min_box.append(&min_spin);
    min_box.set_visible(habit.habit_type == HabitType::Number);

    // Show/hide min_box based on type selection
    {
        let min_box = min_box.clone();
        type_view.connect_changed(move |combo| {
            let idx = combo.active();
            min_box.set_visible(idx == Some(2));
        });
    }

    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);

    content.append(&desc_entry);
    content.append(&type_box);
    content.append(&min_box);
    content.append(&btn_box);
    dialog.set_child(Some(&content));

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog.close();
        });
    }

    {
        let db = db.clone();
        let habits = habits.clone();
        let habit_list = habit_list.clone();
        let wsl = wsl.clone();
        let dialog = dialog.clone();
        let habit_id = habit.id;
        let timer_shared = timer_shared.clone();
        let timer_label = timer_label.clone();
        let timer_btn = timer_btn.clone();
        let settings = settings.clone();
        let window = window.clone();

        save_btn.connect_clicked(move |_| {
            let desc = desc_entry.text().to_string();
            if desc.trim().is_empty() {
                return;
            }
            let habit_type = match type_view.active() {
                Some(1) => HabitType::Boolean,
                Some(2) => HabitType::Number,
                _ => HabitType::Timer,
            };
            let min_value = if habit_type == HabitType::Number {
                let mv = min_spin.value() as u32;
                if mv > 0 { Some(mv) } else { None }
            } else {
                None
            };

            let _ = db
                .borrow_mut()
                .update_habit(habit_id, &desc, 1500, HabitMode::Stopwatch, habit_type, min_value);
            dialog.close();
            super::refresh_from_closures_compat(
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

pub fn delete_habit(
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
    let id = habit.id;
    let desc = habit.description.clone();

    let confirm = gtk4::Dialog::new();
    confirm.set_title(Some("Delete Habit"));
    confirm.set_transient_for(Some(window));
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
        let db = db.clone();
        let habits = habits.clone();
        let habit_list = habit_list.clone();
        let wsl = wsl.clone();
        let timer_shared = timer_shared.clone();
        let timer_label = timer_label.clone();
        let timer_btn = timer_btn.clone();
        let settings = settings.clone();
        let window = window.clone();

        let confirm_yes = confirm.clone();
        yes_btn.connect_clicked(move |_| {
            let _ = db.borrow_mut().delete_habit(id);
            let borrowed = timer_shared.borrow();
            let aid = borrowed.active_habit_id.borrow();
            if *aid == Some(id) {
                borrowed.active_habit_id.replace(None);
            }
            confirm_yes.close();
            super::refresh_from_closures_compat(
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
}

/// Show dialog to edit the session time for a habit on a specific date.
pub fn show_edit_session_time(
    window: &gtk4::ApplicationWindow,
    db: &RefCell<Database>,
    settings: &RefCell<Settings>,
    habits: &RefCell<Vec<Habit>>,
    habit_list: &gtk4::Box,
    wsl: &gtk4::Label,
    timer_shared: TimerSharedRef,
    timer_label: &gtk4::Label,
    timer_btn: &gtk4::Button,
    habit_id: i64,
    date: chrono::NaiveDate,
) {
    let dialog = gtk4::Dialog::new();
    dialog.set_title(Some("Edit Session Time"));
    dialog.set_transient_for(Some(window));
    dialog.set_destroy_with_parent(true);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    let current_minutes = db.borrow().get_total_minutes_for_habit_date(habit_id, date).unwrap_or(0);
    let date_str = date.format("%Y-%m-%d").to_string();
    let label = gtk4::Label::new(Some(&format!("Minutes for {}:", date_str)));
    label.set_xalign(0.0);

    let spin = gtk4::SpinButton::with_range(0.0, 1440.0, 1.0);
    spin.set_value(current_minutes as f64);
    spin.set_digits(0);

    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let save_btn = gtk4::Button::with_label("Save");
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    btn_box.append(&spacer);
    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);

    content.append(&label);
    content.append(&spin);
    content.append(&btn_box);

    dialog.set_child(Some(&content));

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog.close();
        });
    }

    let db = db.clone();
    let settings = settings.clone();
    let habits = habits.clone();
    let habit_list = habit_list.clone();
    let wsl = wsl.clone();
    let timer_label = timer_label.clone();
    let timer_btn = timer_btn.clone();
    let window = window.clone();

    {
        let dialog = dialog.clone();
        save_btn.connect_clicked(move |_| {
            let minutes = spin.value() as u32;
            let seconds = minutes * 60;
            let _ = db.borrow_mut().delete_sessions_for_habit_date(habit_id, date);
            if seconds > 0 {
                let now_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = db.borrow_mut().insert_session(habit_id, date, seconds, &now_str);
            }
            dialog.close();
            super::refresh_from_closures_compat(
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

/// Show dialog to set a value for a Number habit on a specific date.
pub fn show_edit_number_value(
    window: &gtk4::ApplicationWindow,
    db: &RefCell<Database>,
    settings: &RefCell<Settings>,
    habits: &RefCell<Vec<Habit>>,
    habit_list: &gtk4::Box,
    wsl: &gtk4::Label,
    timer_shared: TimerSharedRef,
    timer_label: &gtk4::Label,
    timer_btn: &gtk4::Button,
    habit_id: i64,
    date: chrono::NaiveDate,
    min_value: Option<u32>,
) {
    let dialog = gtk4::Dialog::new();
    dialog.set_title(Some("Set Value"));
    dialog.set_transient_for(Some(window));
    dialog.set_destroy_with_parent(true);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    let current_val = db.borrow().get_value_for_habit_date(habit_id, date).unwrap_or(None).unwrap_or(0.0);
    let date_str = date.format("%Y-%m-%d").to_string();

    let mut label_text = format!("Value for {}:", date_str);
    if let Some(min) = min_value {
        label_text.push_str(&format!(" (min: {})", min));
    }
    let label = gtk4::Label::new(Some(&label_text));
    label.set_xalign(0.0);

    let spin = gtk4::SpinButton::with_range(0.0, 100000.0, 1.0);
    spin.set_value(current_val);
    spin.set_digits(0);

    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let save_btn = gtk4::Button::with_label("Save");
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    btn_box.append(&spacer);
    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);

    content.append(&label);
    content.append(&spin);
    content.append(&btn_box);

    dialog.set_child(Some(&content));

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog.close();
        });
    }

    let db = db.clone();
    let settings = settings.clone();
    let habits = habits.clone();
    let habit_list = habit_list.clone();
    let wsl = wsl.clone();
    let timer_label = timer_label.clone();
    let timer_btn = timer_btn.clone();
    let window = window.clone();

    {
        let dialog = dialog.clone();
        save_btn.connect_clicked(move |_| {
            let val = spin.value();
            let _ = db.borrow_mut().delete_sessions_for_habit_date(habit_id, date);
            let _ = db.borrow_mut().set_value_for_habit_date(habit_id, date, val);
            dialog.close();
            super::refresh_from_closures_compat(
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


