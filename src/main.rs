mod config;
mod parser;
mod ui;

use libadwaita as adw;
use adw::prelude::*;
use ui::App;
use gtk4 as gtk;

use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::Context;

fn install_locally() -> anyhow::Result<()> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let home_path = Path::new(&home);
    let bin_dir = home_path.join(".local/bin");
    let app_dir = home_path.join(".local/share/applications");
    let icon_dir = home_path.join(".local/share/icons/hicolor/48x48/apps");
    let mime_dir = home_path.join(".local/share/mime/packages");

    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&app_dir)?;
    fs::create_dir_all(&icon_dir)?;
    fs::create_dir_all(&mime_dir)?;

    let current_exe = std::env::current_exe()?;
    fs::copy(&current_exe, bin_dir.join("sfmde"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(bin_dir.join("sfmde"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(bin_dir.join("sfmde"), perms)?;
    }

    if Path::new("com.sascha.SFMDE.desktop").exists() {
        fs::copy("com.sascha.SFMDE.desktop", app_dir.join("com.sascha.SFMDE.desktop"))?;
    }

    let icon_src = if Path::new("logo.png").exists() {
        Some(Path::new("logo.png"))
    } else if Path::new("res/icons/logo.png").exists() {
        Some(Path::new("res/icons/logo.png"))
    } else {
        None
    };

    if let Some(src) = icon_src {
        fs::copy(src, icon_dir.join("com.sascha.SFMDE.png"))?;
    }

    // Install MIME type definition so .smd files are recognised
    let mime_xml = mime_dir.join("smd.xml");
    let mime_content = r#"<?xml version="1.0" encoding="utf-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="text/x-smd">
    <comment>Sascha Flavored Markdown document</comment>
    <glob pattern="*.smd"/>
    <icon name="text-x-generic"/>
  </mime-type>
</mime-info>
"#;
    fs::write(&mime_xml, mime_content)?;

    // Refresh MIME and desktop databases
    let _ = std::process::Command::new("update-mime-database")
        .arg(home_path.join(".local/share/mime"))
        .status();
    let _ = std::process::Command::new("update-desktop-database")
        .arg(&app_dir)
        .status();
    let _ = std::process::Command::new("xdg-mime")
        .args(["default", "com.sascha.SFMDE.desktop", "text/x-smd"])
        .status();

    Ok(())
}

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
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|_app| {
        gtk::IconTheme::for_display(&gtk::gdk::Display::default().unwrap())
            .add_resource_path("/com/sascha/SFMDE/icons");
    });

    // Shared handle so connect_open can access the running App
    let shared_ui: Rc<RefCell<Option<App>>> = Rc::new(RefCell::new(None));
    let shared_ui_open = shared_ui.clone();

    app.connect_activate(move |app| {
        let ui = App::new(app);
        ui.window.set_icon_name(Some("logo"));
        ui.window.present();

        if first_run {
            show_welcome_dialog(&ui.window);
        }

        *shared_ui.borrow_mut() = Some(ui);
    });

    app.connect_open(move |_app, files, _hint| {
        if let Some(ui) = shared_ui_open.borrow().as_ref() {
            if let Some(file) = files.first() {
                if let Some(path) = file.path() {
                    ui.open_file(&path);
                    ui.window.present();
                }
            }
        }
    });

    app.run();
}

fn show_welcome_dialog(parent: &adw::ApplicationWindow) {
    let window = adw::Window::builder()
        .title("Welcome to SFMDE")
        .modal(true)
        .transient_for(parent)
        .default_width(500)
        .build();
    window.add_css_class("dialog-border");

    let status_page = adw::StatusPage::builder()
        .icon_name("com.sascha.SFMDE")
        .title("Welcome to SFMDE")
        .description(format!("Version {}\n\nSascha Flavored Markdown (SFM) is a highly customizable superset of Markdown.\n\nEvery formatting symbol—from bold to spoilers—can be redefined in your configuration file. This allows you to tailor the editor to your preferred syntax while maintaining live previews and standard Markdown features.\n\nCheck your configuration at ~/.config/sascha-flavored-markdown/sfmde.config to start customizing.", env!("CARGO_PKG_VERSION")))
        .build();
    status_page.set_margin_top(16);
    status_page.set_margin_bottom(24);
    status_page.set_margin_start(16);
    status_page.set_margin_end(16);

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    btn_box.set_halign(gtk::Align::Center);
    btn_box.set_margin_top(16);
    btn_box.set_margin_bottom(8);

    let install_btn = gtk::Button::builder()
        .label("Install Locally")
        .css_classes(["suggested-action"])
        .build();

    let start_btn = gtk::Button::with_label("Let's go!");

    btn_box.append(&install_btn);
    btn_box.append(&start_btn);

    status_page.set_child(Some(&btn_box));
    window.set_content(Some(&status_page));

    let window_clone = window.clone();
    start_btn.connect_clicked(move |_| {
        window_clone.close();
    });

    let window_install_clone = window.clone();
    install_btn.connect_clicked(move |_| {
        match install_locally() {
            Ok(_) => {
                let success_dialog = adw::AlertDialog::builder()
                    .heading("Installed")
                    .body("SFMDE has been installed to ~/.local/bin, a desktop entry created, and the .smd MIME type registered. You may now double-click .smd files to open them.")
                    .build();
                success_dialog.add_css_class("dialog-border");

                let icon = gtk::Image::from_icon_name("com.sascha.SFMDE");
                icon.set_pixel_size(64);
                success_dialog.set_extra_child(Some(&icon));

                success_dialog.add_response("ok", "Great!");
                success_dialog.choose(Some(&window_install_clone), gtk::gio::Cancellable::NONE, |_| {});
            }
            Err(e) => {
                let error_dialog = adw::AlertDialog::builder()
                    .heading("Installation Failed")
                    .body(format!("Could not install SFMDE: {}", e))
                    .build();
                error_dialog.add_css_class("dialog-border");

                let icon = gtk::Image::from_icon_name("com.sascha.SFMDE");
                icon.set_pixel_size(64);
                error_dialog.set_extra_child(Some(&icon));

                error_dialog.add_response("ok", "OK");
                error_dialog.choose(Some(&window_install_clone), gtk::gio::Cancellable::NONE, |_| {});
            }
        }
    });

    window.present();
}
