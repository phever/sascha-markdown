mod config;
mod parser;
mod ui;

use libadwaita as adw;
use adw::prelude::*;
use ui::App;
use gtk4 as gtk;
use gtk::prelude::*;

fn main() {
    // Register resources
    gio::resources_register_include!("resources.gresource").expect("Failed to register resources");

    let first_run = match config::ensure_config_exists() {
        Ok(created) => created,
        Err(e) => {
            eprintln!("Warning: Could not initialize global config: {}", e);
            false
        }
    };

    let app = adw::Application::builder()
        .application_id("com.sascha.SFMDE")
        .build();

    app.connect_startup(|app| {
        gtk::IconTheme::for_display(&gtk::gdk::Display::default().unwrap())
            .add_resource_path("/com/sascha/SFMDE/icons");
    });

    app.connect_activate(move |app| {
        let ui = App::new(app);
        ui.window.set_icon_name(Some("logo"));
        ui.window.present();

        if first_run {
            let dialog = adw::AlertDialog::builder()
                .heading("Welcome First-Time User!")
                .body("Sascha Flavored Markdown (SFM) is a highly customizable superset of Markdown.\n\nEvery formatting symbol—from bold to spoilers—can be redefined in your configuration file. This allows you to tailor the editor to your preferred syntax while maintaining live previews and standard Markdown features.\n\nCheck your configuration at ~/.config/sascha-flavored-markdown/sfmde.config to start customizing.")
                .build();
            dialog.add_response("ok", "Let's go!");
            dialog.set_default_response(Some("ok"));
            dialog.add_css_class("dialog-border");
            dialog.choose(Some(&ui.window), gtk::gio::Cancellable::NONE, move |_| {});
        }
    });

    app.run();
}
