use gtk4 as gtk;
use gtk::glib;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ui::AppState;
use crate::ui::toolbar::refresh_toolbar;
use crate::ui::app::{apply_appearance, setup_accels};

pub fn show_settings_dialog(parent: &adw::ApplicationWindow, state: Rc<RefCell<AppState>>) {
    let dialog = adw::PreferencesWindow::builder()
        .title("Settings")
        .transient_for(parent)
        .modal(true)
        .build();

    let user_page = adw::PreferencesPage::new();
    user_page.set_title("User");
    user_page.set_icon_name(Some("avatar-default-symbolic"));
    dialog.add(&user_page);

    let local_page = adw::PreferencesPage::new();
    local_page.set_title("Local");
    local_page.set_icon_name(Some("folder-symbolic"));
    dialog.add(&local_page);

    let hotkeys_page = adw::PreferencesPage::new();
    hotkeys_page.set_title("Hotkeys");
    hotkeys_page.set_icon_name(Some("preferences-desktop-keyboard-shortcuts-symbolic"));
    dialog.add(&hotkeys_page);

    let appearance_page = adw::PreferencesPage::new();
    appearance_page.set_title("Appearance");
    appearance_page.set_icon_name(Some("preferences-desktop-appearance-symbolic"));
    dialog.add(&appearance_page);

    let user_group = adw::PreferencesGroup::new();
    user_group.set_title("Formatters (Global)");
    user_page.add(&user_group);

    let local_group = adw::PreferencesGroup::new();
    local_group.set_title("Formatters (Local)");
    local_page.add(&local_group);

    let hotkeys_group = adw::PreferencesGroup::new();
    hotkeys_group.set_title("Keyboard Shortcuts");
    hotkeys_page.add(&hotkeys_group);

    let appearance_group = adw::PreferencesGroup::new();
    appearance_group.set_title("Editor Appearance");
    appearance_page.add(&appearance_group);

    let global_config = crate::config::get_global_config_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|c| toml::from_str::<crate::config::Config>(&c).ok())
        .unwrap_or_default();

    let local_config = if let Some(path) = &state.borrow().current_file {
        crate::config::load_config(path.parent().unwrap()).unwrap_or_default()
    } else {
        crate::config::Config::default()
    };

    populate_config_group(&user_group, global_config.clone(), true, state.clone());
    populate_config_group(&local_group, local_config, false, state.clone());
    populate_hotkeys_group(&hotkeys_group, global_config.clone(), state.clone());
    populate_appearance_group(&appearance_group, global_config, state.clone());

    dialog.present();
}

pub fn populate_hotkeys_group(group: &adw::PreferencesGroup, config: crate::config::Config, state: Rc<RefCell<AppState>>) {
    let mut formatter_names = Vec::new();
    for (name, _, _, _) in config.formatters.all_formatters() {
        formatter_names.push(name);
    }
    
    let hotkeys = config.hotkeys.all_hotkeys(&formatter_names);
    for (name, shortcut) in hotkeys {
        let row = adw::ActionRow::new();
        row.set_title(&name);

        let entry = gtk::Entry::new();
        entry.set_text(&shortcut);
        entry.set_valign(gtk::Align::Center);
        row.add_suffix(&entry);

        group.add(&row);

        let state_clone = state.clone();
        let name_clone = name.clone();
        let entry_clone = entry.clone();
        
        entry.connect_changed(move |_| {
            let new_shortcut = entry_clone.text().to_string();
            
            let mut current_config = crate::config::get_global_config_path()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|c| toml::from_str::<crate::config::Config>(&c).ok())
                .unwrap_or_default();

            current_config.hotkeys.update(&name_clone, new_shortcut);
            let _ = crate::config::save_global_config(&current_config);

            // Update app state and accels
            let mut s = state_clone.borrow_mut();
            s.config.hotkeys = current_config.hotkeys.clone();
            
            if let Some(app) = s.toolbar.as_ref().and_then(|t| t.root()).and_then(|r| r.downcast::<gtk::Window>().ok()).and_then(|w| w.application()) {
                setup_accels(&app.downcast::<adw::Application>().unwrap(), &s.config);
            }
        });
    }
}

pub fn populate_appearance_group(group: &adw::PreferencesGroup, config: crate::config::Config, state: Rc<RefCell<AppState>>) {
    // Font Family
    let font_family_row = adw::ActionRow::new();
    font_family_row.set_title("Font Family");
    let font_family_entry = gtk::Entry::new();
    font_family_entry.set_text(&config.appearance.editor_font_family);
    font_family_entry.set_valign(gtk::Align::Center);
    font_family_row.add_suffix(&font_family_entry);
    group.add(&font_family_row);

    // Font Size
    let font_size_row = adw::ActionRow::new();
    font_size_row.set_title("Font Size");
    let font_size_spin = gtk::SpinButton::with_range(8.0, 72.0, 1.0);
    font_size_spin.set_value(config.appearance.editor_font_size as f64);
    font_size_spin.set_valign(gtk::Align::Center);
    font_size_row.add_suffix(&font_size_spin);
    group.add(&font_size_row);

    // Bg Color
    let bg_color_row = adw::ActionRow::new();
    bg_color_row.set_title("Background Color");
    let bg_color_entry = gtk::Entry::new();
    bg_color_entry.set_text(&config.appearance.editor_bg_color);
    bg_color_entry.set_valign(gtk::Align::Center);
    bg_color_entry.set_placeholder_text(Some("e.g. #333333"));
    bg_color_row.add_suffix(&bg_color_entry);
    group.add(&bg_color_row);

    // Fg Color
    let fg_color_row = adw::ActionRow::new();
    fg_color_row.set_title("Foreground Color");
    let fg_color_entry = gtk::Entry::new();
    fg_color_entry.set_text(&config.appearance.editor_fg_color);
    fg_color_entry.set_valign(gtk::Align::Center);
    fg_color_entry.set_placeholder_text(Some("e.g. #FFFFFF"));
    fg_color_row.add_suffix(&fg_color_entry);
    group.add(&fg_color_row);

    // Icon Size
    let icon_size_row = adw::ActionRow::new();
    icon_size_row.set_title("Menu Icon Size");
    let icon_size_spin = gtk::SpinButton::with_range(8.0, 64.0, 1.0);
    icon_size_spin.set_value(config.appearance.menu_icon_size as f64);
    icon_size_spin.set_valign(gtk::Align::Center);
    icon_size_row.add_suffix(&icon_size_spin);
    group.add(&icon_size_row);

    // Splitbar Color
    let split_color_row = adw::ActionRow::new();
    split_color_row.set_title("Split Bar Color");
    let split_color_entry = gtk::Entry::new();
    split_color_entry.set_text(&config.appearance.splitbar_color);
    split_color_entry.set_valign(gtk::Align::Center);
    split_color_row.add_suffix(&split_color_entry);
    group.add(&split_color_row);

    // Splitbar Width
    let split_width_row = adw::ActionRow::new();
    split_width_row.set_title("Split Bar Width (px)");
    let split_width_spin = gtk::SpinButton::with_range(1.0, 20.0, 1.0);
    split_width_spin.set_value(config.appearance.splitbar_width as f64);
    split_width_spin.set_valign(gtk::Align::Center);
    split_width_row.add_suffix(&split_width_spin);
    group.add(&split_width_row);

    let state_clone = state.clone();
    let ff_clone = font_family_entry.clone();
    let fs_clone = font_size_spin.clone();
    let bg_clone = bg_color_entry.clone();
    let fg_clone = fg_color_entry.clone();
    let is_clone = icon_size_spin.clone();
    let sc_clone = split_color_entry.clone();
    let sw_clone = split_width_spin.clone();

    let save_func = move || {
        let mut current_config = crate::config::get_global_config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|c| toml::from_str::<crate::config::Config>(&c).ok())
            .unwrap_or_default();

        current_config.appearance.editor_font_family = ff_clone.text().to_string();
        current_config.appearance.editor_font_size = fs_clone.value() as u32;
        current_config.appearance.editor_bg_color = bg_clone.text().to_string();
        current_config.appearance.editor_fg_color = fg_clone.text().to_string();
        current_config.appearance.menu_icon_size = is_clone.value() as u32;
        current_config.appearance.splitbar_color = sc_clone.text().to_string();
        current_config.appearance.splitbar_width = sw_clone.value() as u32;

        let _ = crate::config::save_global_config(&current_config);

        let mut s = state_clone.borrow_mut();
        s.config.appearance = current_config.appearance.clone();
        apply_appearance(&s.css_provider, &s.config.appearance);
    };

    let s1 = save_func.clone(); font_family_entry.connect_changed(move |_| s1());
    let s2 = save_func.clone(); font_size_spin.connect_value_changed(move |_| s2());
    let s3 = save_func.clone(); bg_color_entry.connect_changed(move |_| s3());
    let s4 = save_func.clone(); fg_color_entry.connect_changed(move |_| s4());
    let s5 = save_func.clone(); split_color_entry.connect_changed(move |_| s5());
    let s6 = save_func.clone(); split_width_spin.connect_value_changed(move |_| s6());
    icon_size_spin.connect_value_changed(move |_| save_func());
}

pub fn populate_config_group(group: &adw::PreferencesGroup, config: crate::config::Config, is_global: bool, state: Rc<RefCell<AppState>>) {
    let formatters = config.formatters.all_formatters();
    for (name, symbol, visible, icon_name) in formatters {
        let row = adw::ActionRow::new();
        row.set_title(&name);

        let icon_img = gtk::Image::from_icon_name(&icon_name);
        row.add_prefix(&icon_img);

        let entry = gtk::Entry::new();
        entry.set_text(&symbol);
        entry.set_valign(gtk::Align::Center);
        entry.set_placeholder_text(Some("Symbol"));
        row.add_suffix(&entry);

        let icon_entry = gtk::Entry::new();
        icon_entry.set_text(&icon_name);
        icon_entry.set_valign(gtk::Align::Center);
        icon_entry.set_placeholder_text(Some("Icon Name"));
        row.add_suffix(&icon_entry);

        let toggle = gtk::Switch::new();
        toggle.set_active(visible);
        toggle.set_valign(gtk::Align::Center);
        row.add_suffix(&toggle);

        group.add(&row);

        let state_clone = state.clone();
        let name_clone = name.clone();
        let is_global_clone = is_global;
        
        let entry_clone = entry.clone();
        let icon_entry_clone = icon_entry.clone();
        let toggle_clone = toggle.clone();
        let icon_img_clone = icon_img.clone();
        
        let save_func = move || {
            let sym = entry_clone.text().to_string();
            let ico = icon_entry_clone.text().to_string();
            let vis = toggle_clone.is_active();
            
            icon_img_clone.set_icon_name(Some(&ico));

            let mut current_config = if is_global_clone {
                 crate::config::get_global_config_path()
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .and_then(|c| toml::from_str::<crate::config::Config>(&c).ok())
                    .unwrap_or_default()
            } else {
                if let Some(path) = &state_clone.borrow().current_file {
                    crate::config::load_config(path.parent().unwrap()).unwrap_or_default()
                } else {
                    crate::config::Config::default()
                }
            };

            let mut f_vec = current_config.formatters.all_formatters();
            for (n, s, v, i) in f_vec.iter_mut() {
                if n == &name_clone {
                    *s = sym.clone();
                    *v = vis;
                    *i = ico.clone();
                }
            }
            current_config.formatters.update_from_vec(f_vec);

            if is_global_clone {
                let _ = crate::config::save_global_config(&current_config);
            } else {
                if let Some(path) = &state_clone.borrow().current_file {
                    let _ = crate::config::save_local_config(&current_config, path.parent().unwrap());
                }
            }

            // Refresh app state if it's the active config
            let mut s = state_clone.borrow_mut();
            if let Some(path) = &s.current_file {
                if let Ok(new_conf) = crate::config::load_config(path.parent().unwrap()) {
                    s.config = new_conf;
                }
            } else {
                if let Ok(new_conf) = crate::config::load_config(&std::env::current_dir().unwrap()) {
                    s.config = new_conf;
                }
            }
            drop(s);
            refresh_toolbar(state_clone.clone(), None);
        };

        let save_clone = save_func.clone();
        entry.connect_changed(move |_| {
            save_clone();
        });

        let save_ico_clone = save_func.clone();
        icon_entry.connect_changed(move |_| {
            save_ico_clone();
        });

        toggle.connect_state_set(move |_, _| {
            save_func();
            glib::Propagation::Proceed
        });
    }
}