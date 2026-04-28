use gtk4::gio;
use gtk4::prelude::*;

use crate::db::Database;
use crate::settings::Settings;
use crate::ui::AppView;

mod db;
mod settings;
mod ui;

fn main() {
    gtk4::init().expect("Failed to initialize GTK");

    let app = gtk4::Application::new(Some("com.pomotasker.app"), gio::ApplicationFlags::empty());

    // Force dark theme
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    app.connect_activate(move |app| {
        // Database
        let db = Database::new("pomotasker.db").expect("Failed to initialize database");

        // Settings
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
