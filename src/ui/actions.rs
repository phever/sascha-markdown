use gtk4 as gtk;
use gtk::gio;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ui::{AppState, App};
use crate::ui::markup::apply_markup;
use crate::ui::settings::show_settings_dialog;
use crate::config::Config;

pub fn setup_accels(app: &adw::Application, config: &Config) {
    // Basic actions
    app.set_accels_for_action("app.save", &[&config.hotkeys.get("Save")]);
    app.set_accels_for_action("app.open", &[&config.hotkeys.get("Open")]);
    app.set_accels_for_action("app.new", &[&config.hotkeys.get("New File")]);
    app.set_accels_for_action("app.undo", &[&config.hotkeys.get("Undo")]);
    app.set_accels_for_action("app.redo", &[&config.hotkeys.get("Redo")]);

    // Formatter actions (dynamic)
    for (name, _, _, _) in config.formatters.all_formatters() {
        let action_name = name.to_lowercase().replace(' ', "-");
        app.set_accels_for_action(&format!("app.{}", action_name), &[&config.hotkeys.get(&name)]);
    }
}

pub fn register_actions(
    app: &adw::Application,
    state: &Rc<RefCell<AppState>>,
    window: &adw::ApplicationWindow,
    settings_btn: &gtk::Button,
) {
    let action_new = gio::SimpleAction::new("new", None);
    let state_an_clone = state.clone();
    action_new.connect_activate(move |_, _| {
        App::create_new_tab(&state_an_clone, None);
    });
    app.add_action(&action_new);

    let action_open = gio::SimpleAction::new("open", None);
    let state_ao_clone = state.clone();
    let window_ao_clone = window.clone();
    action_open.connect_activate(move |_, _| {
        App::trigger_open_dialog(&state_ao_clone, &window_ao_clone);
    });
    app.add_action(&action_open);

    let action_save = gio::SimpleAction::new("save", None);
    let state_as_clone = state.clone();
    let window_as_clone = window.clone();
    action_save.connect_activate(move |_, _| {
        let active_tab = state_as_clone.borrow().get_active_tab();
        if let Some(tab) = active_tab {
            let state_inner = state_as_clone.clone();
            let state_inner_for_cb = state_inner.clone();
            App::save_tab_with_callback(&state_inner, &tab, &window_as_clone, move |success| {
                if success {
                    App::sync_ui_to_active_tab(&state_inner_for_cb);
                }
            });
        }
    });
    app.add_action(&action_save);

    let action_save_as = gio::SimpleAction::new("save-as", None);
    let state_asa_clone = state.clone();
    let window_asa_clone = window.clone();
    action_save_as.connect_activate(move |_, _| {
        let active_tab = state_asa_clone.borrow().get_active_tab();
        if let Some(tab) = active_tab {
            let original_file = tab.borrow().file.clone();
            tab.borrow_mut().file = None;
            
            let state_inner = state_asa_clone.clone();
            let state_inner_for_cb = state_inner.clone();
            let tab_inner = tab.clone();
            let tab_inner_for_cb = tab_inner.clone();
            App::save_tab_with_callback(&state_inner, &tab_inner, &window_asa_clone, move |success| {
                if !success {
                    tab_inner_for_cb.borrow_mut().file = original_file.clone();
                } else {
                    App::sync_ui_to_active_tab(&state_inner_for_cb);
                }
            });
        }
    });
    app.add_action(&action_save_as);

    let action_export_html = gio::SimpleAction::new("export-html", None);
    let state_aeh_clone = state.clone();
    let window_aeh_clone = window.clone();
    action_export_html.connect_activate(move |_, _| {
        let active_tab = state_aeh_clone.borrow().get_active_tab();
        if let Some(tab) = active_tab {
            App::export_tab_as_html(&state_aeh_clone, &tab, &window_aeh_clone);
        }
    });
    app.add_action(&action_export_html);

    let action_undo = gio::SimpleAction::new("undo", None);
    let state_au_clone = state.clone();
    action_undo.connect_activate(move |_, _| {
        if let Some(tab) = state_au_clone.borrow().get_active_tab() {
            let tab_borrow = tab.borrow();
            if tab_borrow.buffer.can_undo() {
                tab_borrow.buffer.undo();
            }
        }
    });
    app.add_action(&action_undo);

    let action_redo = gio::SimpleAction::new("redo", None);
    let state_ar_clone = state.clone();
    action_redo.connect_activate(move |_, _| {
        if let Some(tab) = state_ar_clone.borrow().get_active_tab() {
            let tab_borrow = tab.borrow();
            if tab_borrow.buffer.can_redo() {
                tab_borrow.buffer.redo();
            }
        }
    });
    app.add_action(&action_redo);

    let action_bold = gio::SimpleAction::new("bold", None);
    let state_b_clone = state.clone();
    action_bold.connect_activate(move |_, _| {
        let s = state_b_clone.borrow();
        if let Some(tab) = s.get_active_tab() {
            let tab_borrow = tab.borrow();
            let symbol = s.config.formatters.bold.symbol.clone();
            apply_markup(&tab_borrow.buffer, &symbol);
        }
    });
    app.add_action(&action_bold);

    let action_italics = gio::SimpleAction::new("italics", None);
    let state_i_clone = state.clone();
    action_italics.connect_activate(move |_, _| {
        let s = state_i_clone.borrow();
        if let Some(tab) = s.get_active_tab() {
            let tab_borrow = tab.borrow();
            let symbol = s.config.formatters.italics.symbol.clone();
            apply_markup(&tab_borrow.buffer, &symbol);
        }
    });
    app.add_action(&action_italics);

    let action_underscore = gio::SimpleAction::new("underscore", None);
    let state_u_clone = state.clone();
    action_underscore.connect_activate(move |_, _| {
        let s = state_u_clone.borrow();
        if let Some(tab) = s.get_active_tab() {
            let tab_borrow = tab.borrow();
            let symbol = s.config.formatters.underscore.symbol.clone();
            apply_markup(&tab_borrow.buffer, &symbol);
        }
    });
    app.add_action(&action_underscore);

    let action_strikethrough = gio::SimpleAction::new("strikethrough", None);
    let state_s_clone = state.clone();
    action_strikethrough.connect_activate(move |_, _| {
        let s = state_s_clone.borrow();
        if let Some(tab) = s.get_active_tab() {
            let tab_borrow = tab.borrow();
            let symbol = s.config.formatters.strikethrough.symbol.clone();
            apply_markup(&tab_borrow.buffer, &symbol);
        }
    });
    app.add_action(&action_strikethrough);

    let word_wrap_init = state.borrow().config.appearance.word_wrap;
    let action_word_wrap = gio::SimpleAction::new_stateful(
        "word-wrap",
        None,
        &word_wrap_init.to_variant(),
    );
    let state_ww_clone = state.clone();
    action_word_wrap.connect_activate(move |action, _| {
        let new_val = !action.state().and_then(|v| v.get::<bool>()).unwrap_or(false);
        action.set_state(&new_val.to_variant());
        let mut s = state_ww_clone.borrow_mut();
        s.config.appearance.word_wrap = new_val;
        let _ = crate::config::save_global_config(&s.config);
        
        for tab in &s.open_tabs {
            let tab_borrow = tab.borrow();
            tab_borrow.editor_view.set_wrap_mode(if new_val {
                gtk4::WrapMode::WordChar
            } else {
                gtk4::WrapMode::None
            });
        }
    });
    app.add_action(&action_word_wrap);

    let action_open_recent = gio::SimpleAction::new(
        "open-recent",
        Some(&String::static_variant_type()),
    );
    let state_or_clone = state.clone();
    action_open_recent.connect_activate(move |_, param| {
        if let Some(path_str) = param.and_then(|v| v.get::<String>()) {
            let path = std::path::PathBuf::from(&path_str);
            App::open_file_in_tab(&state_or_clone, &path);
        }
    });
    app.add_action(&action_open_recent);

    let action_about = gio::SimpleAction::new("about", None);
    let window_ab_clone = window.clone();
    action_about.connect_activate(move |_, _| {
        let about = adw::AboutDialog::builder()
            .application_name("SFMDE")
            .application_icon("com.sascha.SFMDE")
            .developer_name("Sascha")
            .version(env!("CARGO_PKG_VERSION"))
            .comments("A highly configurable Markdown editor with GTK4 and custom SFM support.")
            .build();
        about.add_css_class("dialog-border");
        about.present(Some(&window_ab_clone));
    });
    app.add_action(&action_about);

    let state_settings_clone = state.clone();
    let window_settings_clone = window.clone();
    settings_btn.connect_clicked(move |_| {
        show_settings_dialog(&window_settings_clone, state_settings_clone.clone());
    });
}
