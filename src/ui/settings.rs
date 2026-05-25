use gtk4 as gtk;
use gtk::glib;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use sourceview5 as source;
use source::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ui::AppState;
use crate::ui::toolbar::refresh_toolbar;
use crate::ui::app::{apply_appearance, setup_accels};
use gtk::pango;

pub fn show_settings_dialog(parent: &adw::ApplicationWindow, state: Rc<RefCell<AppState>>) {
    let dialog = adw::PreferencesWindow::builder()
        .title("Settings")
        .transient_for(parent)
        .modal(true)
        .build();
    dialog.add_css_class("dialog-border");

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
    // Editor Font (family + size combined via FontDialogButton)
    let font_row = adw::ActionRow::new();
    font_row.set_title("Editor Font");
    font_row.set_subtitle("Family and size for the editor pane");
    let font_dialog = gtk::FontDialog::new();
    let font_btn = gtk::FontDialogButton::new(Some(font_dialog));
    font_btn.set_use_font(true);
    font_btn.set_use_size(true);
    font_btn.set_valign(gtk::Align::Center);
    // Initialise from stored family + size
    let initial_font = format!("{} {}", config.appearance.editor_font_family, config.appearance.editor_font_size);
    font_btn.set_font_desc(&pango::FontDescription::from_string(&initial_font));
    font_row.add_suffix(&font_btn);
    group.add(&font_row);

    // Word Wrap
    let wrap_row = adw::ActionRow::new();
    wrap_row.set_title("Word Wrap");
    wrap_row.set_subtitle("Wrap long lines in the editor");
    let wrap_switch = gtk::Switch::new();
    wrap_switch.set_active(config.appearance.word_wrap);
    wrap_switch.set_valign(gtk::Align::Center);
    wrap_row.add_suffix(&wrap_switch);
    group.add(&wrap_row);

    // Show Line Numbers
    let linenum_row = adw::ActionRow::new();
    linenum_row.set_title("Show Line Numbers");
    linenum_row.set_subtitle("Show the line-number gutter in the editor");
    let linenum_switch = gtk::Switch::new();
    linenum_switch.set_active(config.appearance.show_line_numbers);
    linenum_switch.set_valign(gtk::Align::Center);
    linenum_row.add_suffix(&linenum_switch);
    group.add(&linenum_row);

    // Show Line/Col
    let linecol_row = adw::ActionRow::new();
    linecol_row.set_title("Show Line/Col Status");
    linecol_row.set_subtitle("Show cursor position in the status bar");
    let linecol_switch = gtk::Switch::new();
    linecol_switch.set_active(config.appearance.show_line_col);
    linecol_switch.set_valign(gtk::Align::Center);
    linecol_row.add_suffix(&linecol_switch);
    group.add(&linecol_row);

    // Local-Only Mode
    let local_row = adw::ActionRow::new();
    local_row.set_title("Local-Only Mode");
    local_row.set_subtitle("Block all external (non-file://) network requests in preview");
    let local_switch = gtk::Switch::new();
    local_switch.set_active(config.appearance.local_only);
    local_switch.set_valign(gtk::Align::Center);
    local_row.add_suffix(&local_switch);
    group.add(&local_row);

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

    // Highlight Color
    let hl_row = adw::ActionRow::new();
    hl_row.set_title("Highlight Color");
    hl_row.set_subtitle("CSS color for ==highlighted== text (empty = browser default yellow)");
    let hl_entry = gtk::Entry::new();
    hl_entry.set_text(&config.appearance.highlight_color);
    hl_entry.set_valign(gtk::Align::Center);
    hl_entry.set_placeholder_text(Some("e.g. #ffff00 or lightblue"));
    hl_row.add_suffix(&hl_entry);
    group.add(&hl_row);

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
    let font_btn_clone = font_btn.clone();
    let bg_clone = bg_color_entry.clone();
    let fg_clone = fg_color_entry.clone();
    let is_clone = icon_size_spin.clone();
    let sc_clone = split_color_entry.clone();
    let sw_clone = split_width_spin.clone();
    let wrap_clone = wrap_switch.clone();
    let linenum_clone = linenum_switch.clone();
    let linecol_clone = linecol_switch.clone();
    let hl_clone = hl_entry.clone();
    let local_clone = local_switch.clone();

    let save_func = move || {
        let mut current_config = crate::config::get_global_config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|c| toml::from_str::<crate::config::Config>(&c).ok())
            .unwrap_or_default();

        // Extract family and size from the FontDialogButton's FontDescription
        if let Some(desc) = font_btn_clone.font_desc() {
            if let Some(family) = desc.family() {
                current_config.appearance.editor_font_family = family.to_string();
            }
            let size_pango = desc.size();
            if size_pango > 0 {
                let size_pts = (size_pango / pango::SCALE) as u32;
                if size_pts > 0 {
                    current_config.appearance.editor_font_size = size_pts;
                }
            }
        }

        current_config.appearance.editor_bg_color = bg_clone.text().to_string();
        current_config.appearance.editor_fg_color = fg_clone.text().to_string();
        current_config.appearance.menu_icon_size = is_clone.value() as u32;
        current_config.appearance.splitbar_color = sc_clone.text().to_string();
        current_config.appearance.splitbar_width = sw_clone.value() as u32;
        current_config.appearance.word_wrap = wrap_clone.is_active();
        current_config.appearance.show_line_numbers = linenum_clone.is_active();
        current_config.appearance.show_line_col = linecol_clone.is_active();
        current_config.appearance.highlight_color = hl_clone.text().to_string();
        current_config.appearance.local_only = local_clone.is_active();

        let _ = crate::config::save_global_config(&current_config);

        let mut s = state_clone.borrow_mut();
        s.config.appearance = current_config.appearance.clone();
        apply_appearance(&s.css_provider, &s.config.appearance);

        if let Some(view) = &s.editor_view {
            view.set_wrap_mode(if s.config.appearance.word_wrap {
                gtk4::WrapMode::WordChar
            } else {
                gtk4::WrapMode::None
            });
            view.set_show_line_numbers(s.config.appearance.show_line_numbers);
        }
        if let Some(label) = &s.cursor_label {
            label.set_visible(s.config.appearance.show_line_col);
        }
    };

    let s1 = save_func.clone(); bg_color_entry.connect_changed(move |_| s1());
    let s2 = save_func.clone(); fg_color_entry.connect_changed(move |_| s2());
    let s3 = save_func.clone(); split_color_entry.connect_changed(move |_| s3());
    let s4 = save_func.clone(); split_width_spin.connect_value_changed(move |_| s4());
    let s5 = save_func.clone(); hl_entry.connect_changed(move |_| s5());
    let s6 = save_func.clone(); icon_size_spin.connect_value_changed(move |_| s6());
    let s7 = save_func.clone(); wrap_switch.connect_state_set(move |_, _| { s7(); glib::Propagation::Proceed });
    let s8 = save_func.clone(); linenum_switch.connect_state_set(move |_, _| { s8(); glib::Propagation::Proceed });
    let s9 = save_func.clone(); linecol_switch.connect_state_set(move |_, _| { s9(); glib::Propagation::Proceed });
    let s10 = save_func.clone(); local_switch.connect_state_set(move |_, _| { s10(); glib::Propagation::Proceed });
    font_btn.connect_font_desc_notify(move |_| save_func());
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