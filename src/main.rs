use gtk4::gio;
use gtk4::prelude::*;

use crate::db::Database;
use crate::settings::Settings;
use crate::ui::AppView;

pub mod db;
mod settings;
pub mod service;
mod ui;

fn main() {
    gtk4::init().expect("Failed to initialize GTK");

    let app = gtk4::Application::new(Some("com.pomotasker.app"), gio::ApplicationFlags::empty());

    // Force dark theme
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    // Resolve absolute paths so clone() works regardless of CWD
    let mut db_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("XDG_DATA_HOME"))
        .unwrap_or_else(|_| "/home/andrew/.pomotasker".to_string());
    if !db_dir.ends_with(".pomotasker") {
        db_dir = format!("{}/.pomotasker", db_dir);
    }
    std::fs::create_dir_all(&db_dir).expect("Failed to create data dir");
    let db_path = std::path::Path::new(&db_dir).join("pomotasker.db");

    app.connect_activate(move |app| {
        // Database (absolute path)
        let db = Database::new(db_path.to_str().expect("db path"))
            .expect("Failed to initialize database");

        // Settings (will use absolute path internally)
        let settings = Settings::load();

        // Window
        let window = gtk4::ApplicationWindow::new(app);
        window.set_title(Some("PomoTasker"));
        window.set_default_size(340, 700);

        // Build UI
        let view = AppView::new(&window, &db, &settings);
        window.set_child(Some(view.root()));
        window.present();
    });

    app.run();
}
