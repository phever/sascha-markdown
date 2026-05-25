use gtk4 as gtk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use sourceview5 as source;
use source::prelude::*;
use webkit6::prelude::*;
use crate::config::Config;
use std::cell::RefCell;
use std::rc::Rc;
use crate::ui::{AppState, NavState, TabState, App};
use crate::ui::markup::apply_markup;
use crate::ui::toolbar::refresh_toolbar;
use crate::ui::settings::show_settings_dialog;
use std::path::PathBuf;

fn rebuild_recents_menu(state: &Rc<RefCell<AppState>>) {
    let s = state.borrow();
    if let Some(menu) = &s.recents_menu {
        while menu.n_items() > 0 {
            menu.remove(0);
        }
        for path in s.config.recent_files.iter().take(3) {
            let label = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path);
            let item = gio::MenuItem::new(Some(label), None);
            item.set_action_and_target_value(
                Some("app.open-recent"),
                Some(&path.to_variant()),
            );
            menu.append_item(&item);
        }
    }
}

pub fn apply_appearance(provider: &gtk::CssProvider, config: &crate::config::AppearanceConfig) {
    let css = format!("
        textview {{
            font-family: \"{}\";
            font-size: {}pt;
            {}
            {}
        }}
        button image {{
            -gtk-icon-size: {}px;
        }}
        paned > separator {{
            background-color: {};
            min-width: {}px;
            min-height: {}px;
        }}
        .dialog-border {{
            outline: 2px solid rgba(0,0,0,0.35);
            outline-offset: -2px;
        }}
        tabbar {{
            min-height: 30px;
        }}
        tabbar tab {{
            min-height: 30px;
            padding-top: 0px;
            padding-bottom: 0px;
        }}
    ",
        config.editor_font_family,
        config.editor_font_size,
        if config.editor_bg_color.is_empty() { String::new() } else { format!("background-color: {};", config.editor_bg_color) },
        if config.editor_fg_color.is_empty() { String::new() } else { format!("color: {};", config.editor_fg_color) },
        config.menu_icon_size,
        config.splitbar_color,
        config.splitbar_width,
        config.splitbar_width
    );
    provider.load_from_data(&css);
}

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

impl App {
    pub fn new(app: &adw::Application) -> Self {
        let config = if let Ok(dir) = std::env::current_dir() {
            crate::config::load_config(&dir).unwrap_or_default()
        } else {
            Config::default()
        };

        let css_provider = gtk::CssProvider::new();
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        apply_appearance(&css_provider, &config.appearance);

        setup_accels(app, &config);

        // Load custom CSS
        if let Some(css_path) = crate::config::get_style_css_path() {
            if let Ok(css) = std::fs::read_to_string(css_path) {
                let provider = gtk::CssProvider::new();
                provider.load_from_data(&css);
                if let Some(display) = gtk::gdk::Display::default() {
                    gtk::style_context_add_provider_for_display(
                        &display,
                        &provider,
                        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                }
            }
        }

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .default_width(1000)
            .default_height(700)
            .title("SFMDE - Sascha Flavored Markdown Editor")
            .build();

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window.set_content(Some(&main_box));

        let header = adw::HeaderBar::new();
        main_box.append(&header);

        // Undo/Redo (left side, where nav was)
        let undo_redo_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        undo_redo_box.add_css_class("linked");
        header.pack_start(&undo_redo_box);

        let undo_btn = gtk::Button::from_icon_name("edit-undo-symbolic");
        undo_btn.set_tooltip_text(Some("Undo"));
        undo_btn.set_sensitive(false);
        undo_redo_box.append(&undo_btn);

        let redo_btn = gtk::Button::from_icon_name("edit-redo-symbolic");
        redo_btn.set_tooltip_text(Some("Redo"));
        redo_btn.set_sensitive(false);
        undo_redo_box.append(&redo_btn);

        // File buttons
        let new_btn = gtk::Button::from_icon_name("document-new-symbolic");
        new_btn.set_tooltip_text(Some("New File"));
        header.pack_start(&new_btn);

        let open_btn = gtk::Button::with_label("Open");
        header.pack_start(&open_btn);

        let save_btn = gtk::Button::with_label("Save");
        save_btn.set_sensitive(false);
        header.pack_start(&save_btn);

        // Local-Only toggle (right side)
        let local_only_btn = gtk::ToggleButton::builder()
            .icon_name("network-offline-symbolic")
            .active(config.appearance.local_only)
            .tooltip_text("Local-Only Mode: block external network requests in preview")
            .build();
        header.pack_end(&local_only_btn);

        let settings_btn = gtk::Button::from_icon_name("emblem-system-symbolic");
        settings_btn.set_tooltip_text(Some("Settings"));
        header.pack_end(&settings_btn);

        let toggle_editor_btn = gtk::ToggleButton::builder()
            .icon_name("text-x-generic-symbolic")
            .active(true)
            .tooltip_text("Show/Hide Editor")
            .build();
        header.pack_end(&toggle_editor_btn);

        let toggle_preview_btn = gtk::ToggleButton::builder()
            .icon_name("view-preview-symbolic")
            .active(false)
            .tooltip_text("Show/Hide Preview")
            .build();
        header.pack_end(&toggle_preview_btn);

        let preview_mode_btn = gtk::Button::builder()
            .icon_name("display-brightness-symbolic")
            .tooltip_text("Preview Theme: System")
            .build();
        header.pack_end(&preview_mode_btn);

        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        header.pack_end(&menu_button);

        // Create Menu
        let menu_model = gio::Menu::new();
        menu_model.append(Some("New File"), Some("app.new"));
        menu_model.append(Some("Save As..."), Some("app.save-as"));

        // Recent files submenu (rebuilt whenever a file is opened)
        let recents_menu = gio::Menu::new();
        let recents_section = gio::Menu::new();
        let recents_submenu_item = gio::MenuItem::new_submenu(Some("Open Recent"), &recents_menu);
        recents_section.append_item(&recents_submenu_item);
        menu_model.append_section(None, &recents_section);

        let view_section = gio::Menu::new();
        view_section.append(Some("Word Wrap"), Some("app.word-wrap"));
        menu_model.append_section(None, &view_section);

        let section = gio::Menu::new();
        section.append(Some("About SFMDE"), Some("app.about"));
        menu_model.append_section(None, &section);

        menu_button.set_menu_model(Some(&menu_model));

        // Toolbar (Adaptive)
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        toolbar.set_hexpand(true);
        toolbar.set_margin_start(10);
        toolbar.set_margin_end(10);
        toolbar.set_margin_top(5);
        toolbar.set_margin_bottom(5);
        main_box.append(&toolbar);

        // Stack to swap between Welcome Screen and Tabbed Editor
        let main_stack = gtk::Stack::builder()
            .vexpand(true)
            .transition_type(gtk::StackTransitionType::Crossfade)
            .build();
        main_box.append(&main_stack);

        // Child 1: Empty Screen
        let welcome_page = adw::StatusPage::builder()
            .icon_name("com.sascha.SFMDE")
            .title("SFMDE")
            .description("Sascha Flavored Markdown Editor\n\nNo files open.")
            .build();
        
        let welcome_btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        welcome_btn_box.set_halign(gtk::Align::Center);
        
        let welcome_new_btn = gtk::Button::builder()
            .label("New File")
            .css_classes(["suggested-action"])
            .build();
        let welcome_open_btn = gtk::Button::with_label("Open File...");

        welcome_btn_box.append(&welcome_new_btn);
        welcome_btn_box.append(&welcome_open_btn);
        welcome_page.set_child(Some(&welcome_btn_box));
        main_stack.add_named(&welcome_page, Some("empty"));

        // Child 2: Editor tabbed interface
        let editor_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let tab_bar = adw::TabBar::new();
        let tab_view = adw::TabView::new();
        tab_bar.set_view(Some(&tab_view));
        editor_box.append(&tab_bar);
        editor_box.append(&tab_view);
        tab_view.set_vexpand(true);
        main_stack.add_named(&editor_box, Some("editor"));

        main_stack.set_visible_child_name("empty");

        // AppState
        let state = Rc::new(RefCell::new(AppState {
            config: config.clone(),
            toolbar: Some(toolbar.clone()),
            cursor_label: None,
            preview_toggle: Some(toggle_preview_btn.clone()),
            editor_toggle: Some(toggle_editor_btn.clone()),
            undo_btn: Some(undo_btn.clone()),
            redo_btn: Some(redo_btn.clone()),
            save_btn: Some(save_btn.clone()),
            local_only_btn: Some(local_only_btn.clone()),
            preview_color_scheme: 0,
            css_provider,
            recents_menu: Some(recents_menu),
            tab_view: tab_view.clone(),
            tab_bar,
            main_stack,
            open_tabs: Vec::new(),
        }));

        rebuild_recents_menu(&state);

        // Adaptive overflow polling
        let state_poll_clone = state.clone();
        let current_count = Rc::new(RefCell::new(0usize));
        let window_poll_clone = window.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            let width = window_poll_clone.width();
            let button_width_estimate = 42; 
            let primary_count = ((width - 150) / button_width_estimate).max(1) as usize;
            
            if primary_count != *current_count.borrow() {
                *current_count.borrow_mut() = primary_count;
                refresh_toolbar(state_poll_clone.clone(), Some(width - 100));
            }
            glib::ControlFlow::Continue
        });

        // Setup File Callbacks
        let state_new_btn_clone = state.clone();
        new_btn.connect_clicked(move |_| {
            App::create_new_tab(&state_new_btn_clone, None);
        });

        let state_open_btn_clone = state.clone();
        let window_open_btn_clone = window.clone();
        open_btn.connect_clicked(move |_| {
            App::trigger_open_dialog(&state_open_btn_clone, &window_open_btn_clone);
        });

        let state_save_btn_clone = state.clone();
        let window_save_btn_clone = window.clone();
        save_btn.connect_clicked(move |_| {
            let active_tab = state_save_btn_clone.borrow().get_active_tab();
            if let Some(tab) = active_tab {
                let state_inner = state_save_btn_clone.clone();
                let state_inner_for_cb = state_inner.clone();
                App::save_tab_with_callback(&state_inner, &tab, &window_save_btn_clone, move |success| {
                    if success {
                        App::sync_ui_to_active_tab(&state_inner_for_cb);
                    }
                });
            }
        });

        // Welcome screen button clicks
        let state_welcome_new = state.clone();
        welcome_new_btn.connect_clicked(move |_| {
            App::create_new_tab(&state_welcome_new, None);
        });

        let state_welcome_open = state.clone();
        let window_welcome_open = window.clone();
        welcome_open_btn.connect_clicked(move |_| {
            App::trigger_open_dialog(&state_welcome_open, &window_welcome_open);
        });

        // Undo/Redo button clicks
        let state_undo_btn_clone = state.clone();
        undo_btn.connect_clicked(move |_| {
            if let Some(tab) = state_undo_btn_clone.borrow().get_active_tab() {
                let tab_borrow = tab.borrow();
                if tab_borrow.buffer.can_undo() {
                    tab_borrow.buffer.undo();
                }
            }
        });

        let state_redo_btn_clone = state.clone();
        redo_btn.connect_clicked(move |_| {
            if let Some(tab) = state_redo_btn_clone.borrow().get_active_tab() {
                let tab_borrow = tab.borrow();
                if tab_borrow.buffer.can_redo() {
                    tab_borrow.buffer.redo();
                }
            }
        });

        // Toggle Visibility buttons
        let state_editor_toggle = state.clone();
        toggle_editor_btn.connect_toggled(move |btn| {
            let visible = btn.is_active();
            let s = state_editor_toggle.borrow();
            if let Some(tab) = s.get_active_tab() {
                let tab_borrow = tab.borrow();
                if !visible && !tab_borrow.preview.is_visible() {
                    btn.set_active(true);
                } else {
                    tab_borrow.editor_scroll.set_visible(visible);
                }
            }
        });

        let state_preview_toggle = state.clone();
        toggle_preview_btn.connect_toggled(move |btn| {
            let visible = btn.is_active();
            let s = state_preview_toggle.borrow();
            if let Some(tab) = s.get_active_tab() {
                let tab_borrow = tab.borrow();
                if !visible && !tab_borrow.editor_scroll.is_visible() {
                    btn.set_active(true);
                } else {
                    tab_borrow.preview.set_visible(visible);
                    if visible {
                        let paned = &tab_borrow.paned;
                        let pos = paned.position();
                        let width = paned.width();
                        if pos <= 0 || (width > 0 && pos >= width - 50) {
                            paned.set_position(width / 2);
                            if width == 0 {
                                paned.set_position(500);
                            }
                        }
                    }
                }
            }
        });

        // Theme preview click
        let style_manager = adw::StyleManager::default();
        let style_manager_mode_clone = style_manager.clone();
        let preview_mode_btn_clone = preview_mode_btn.clone();
        let state_mode_clone = state.clone();
        
        let update_scheme = {
            let state_scheme = state.clone();
            let scheme_manager = source::StyleSchemeManager::default();
            move |dark: bool| {
                let s = state_scheme.borrow();
                let scheme_id = if dark { "adwaita-dark" } else { "adwaita" };
                let fallback = if dark { "classic-dark" } else { "classic" };
                
                let scheme = scheme_manager.scheme(scheme_id)
                    .or_else(|| scheme_manager.scheme(fallback));

                for tab in &s.open_tabs {
                    let tab_borrow = tab.borrow();
                    if let Some(ref sc) = scheme {
                        tab_borrow.buffer.set_style_scheme(Some(sc));
                    }
                    use glib::prelude::*;
                    tab_borrow.buffer.emit_by_name::<()>("changed", &[]);
                }
            }
        };

        let update_scheme_mode_clone = update_scheme.clone();
        preview_mode_btn.connect_clicked(move |_| {
            let (icon, tooltip) = {
                let mut s = state_mode_clone.borrow_mut();
                s.preview_color_scheme = (s.preview_color_scheme + 1) % 3;
                match s.preview_color_scheme {
                    1 => ("display-brightness-symbolic", "Preview Theme: Light"),
                    2 => ("display-brightness-symbolic", "Preview Theme: Dark"),
                    _ => ("display-brightness-symbolic", "Preview Theme: System"),
                }
            };
            preview_mode_btn_clone.set_icon_name(icon);
            preview_mode_btn_clone.set_tooltip_text(Some(tooltip));

            update_scheme_mode_clone(style_manager_mode_clone.is_dark());
        });

        // Local-Only toggle
        let state_lo_clone = state.clone();
        local_only_btn.connect_toggled(move |btn| {
            let active = btn.is_active();
            let mut s = state_lo_clone.borrow_mut();
            s.config.appearance.local_only = active;
            let _ = crate::config::save_global_config(&s.config);
            
            // Re-render previews to include/remove CSP policy
            for tab in &s.open_tabs {
                use glib::prelude::*;
                tab.borrow().buffer.emit_by_name::<()>("changed", &[]);
            }
        });

        // Status Bar
        let status_bar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        status_bar.set_margin_start(10);
        status_bar.set_margin_end(10);
        status_bar.set_margin_top(5);
        status_bar.set_margin_bottom(5);
        main_box.append(&status_bar);

        let cursor_label = gtk::Label::new(Some("Line: 1, Col: 1"));
        cursor_label.set_visible(state.borrow().config.appearance.show_line_col);
        status_bar.append(&cursor_label);
        state.borrow_mut().cursor_label = Some(cursor_label.clone());

        // Tab selection change notification
        let state_notify_clone = state.clone();
        tab_view.connect_notify_local(Some("selected-page"), move |_, _| {
            App::sync_ui_to_active_tab(&state_notify_clone);
        });

        // Intercept close tab
        let state_close_page_clone = state.clone();
        tab_view.connect_close_page(move |view, page| {
            let state_clone = state_close_page_clone.clone();
            let view_clone = view.clone();
            let page_clone = page.clone();
            
            let tab = {
                let s = state_clone.borrow();
                s.open_tabs.iter().find(|t| t.borrow().tab_page == page_clone).cloned()
            };

            if let Some(tab) = tab {
                let is_dirty = tab.borrow().is_dirty;
                if is_dirty {
                    let parent_window = view_clone.root().and_then(|r| r.downcast::<gtk::Window>().ok());
                    let dlg = adw::Window::builder()
                        .modal(true)
                        .transient_for(parent_window.as_ref().unwrap())
                        .default_width(360)
                        .resizable(false)
                        .title("Unsaved Changes")
                        .build();
                    dlg.add_css_class("dialog-border");

                    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
                    let hbar = adw::HeaderBar::new();
                    hbar.set_show_end_title_buttons(false);
                    vbox.append(&hbar);

                    let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
                    body.set_margin_top(12);
                    body.set_margin_bottom(24);
                    body.set_margin_start(24);
                    body.set_margin_end(24);

                    let heading = gtk::Label::new(Some("Save changes before closing?"));
                    heading.add_css_class("title-3");
                    heading.set_halign(gtk::Align::Start);
                    body.append(&heading);

                    let filename = tab.borrow().file.as_ref()
                        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                        .unwrap_or_else(|| "Untitled".to_string());
                    let msg = gtk::Label::new(Some(&format!("The document \"{}\" has unsaved changes. If you close without saving, your changes will be discarded.", filename)));
                    msg.set_wrap(true);
                    msg.set_halign(gtk::Align::Start);
                    msg.add_css_class("dim-label");
                    body.append(&msg);

                    let btn_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                    btn_row.set_halign(gtk::Align::End);
                    btn_row.set_margin_top(8);
                    let cancel_btn = gtk::Button::with_label("Cancel");
                    let discard_btn = gtk::Button::with_label("Discard");
                    discard_btn.add_css_class("destructive-action");
                    let save_btn_inner = gtk::Button::with_label("Save");
                    save_btn_inner.add_css_class("suggested-action");
                    
                    btn_row.append(&cancel_btn);
                    btn_row.append(&discard_btn);
                    btn_row.append(&save_btn_inner);
                    body.append(&btn_row);

                    vbox.append(&body);
                    dlg.set_content(Some(&vbox));

                    let dlg_cancel = dlg.clone();
                    let view_cancel = view_clone.clone();
                    let page_cancel = page_clone.clone();
                    cancel_btn.connect_clicked(move |_| {
                        view_cancel.close_page_finish(&page_cancel, false);
                        dlg_cancel.close();
                    });

                    let dlg_discard = dlg.clone();
                    let view_discard = view_clone.clone();
                    let page_discard = page_clone.clone();
                    let state_discard = state_clone.clone();
                    discard_btn.connect_clicked(move |_| {
                        state_discard.borrow_mut().open_tabs.retain(|t| t.borrow().tab_page != page_discard);
                        view_discard.close_page_finish(&page_discard, true);
                        App::sync_ui_to_active_tab(&state_discard);
                        dlg_discard.close();
                    });

                    let dlg_save = dlg.clone();
                    let view_save = view_clone.clone();
                    let page_save = page_clone.clone();
                    let state_save = state_clone.clone();
                    let tab_save = tab.clone();
                    save_btn_inner.connect_clicked(move |_| {
                        let state_inner = state_save.clone();
                        let state_inner_for_cb = state_inner.clone();
                        let tab_inner = tab_save.clone();
                        let view_inner = view_save.clone();
                        let page_inner = page_save.clone();
                        let dlg_inner = dlg_save.clone();
                        let dlg_inner_for_cb = dlg_inner.clone();
                        App::save_tab_with_callback(&state_inner, &tab_inner, &dlg_inner, move |success| {
                            if success {
                                state_inner_for_cb.borrow_mut().open_tabs.retain(|t| t.borrow().tab_page != page_inner);
                                view_inner.close_page_finish(&page_inner, true);
                                App::sync_ui_to_active_tab(&state_inner_for_cb);
                                dlg_inner_for_cb.close();
                            }
                        });
                    });

                    dlg.present();
                } else {
                    // Safe to close immediately
                    state_clone.borrow_mut().open_tabs.retain(|t| t.borrow().tab_page != page_clone);
                    view_clone.close_page_finish(&page_clone, true);
                    App::sync_ui_to_active_tab(&state_clone);
                }
            } else {
                view_clone.close_page_finish(&page_clone, true);
            }
            
            glib::Propagation::Stop
        });

        // Register Actions
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

        // Warn on close if there are unsaved changes
        let state_close_clone = state.clone();
        let window_close_clone = window.clone();
        window.connect_close_request(move |_| {
            let any_dirty = state_close_clone.borrow().open_tabs.iter().any(|t| t.borrow().is_dirty);
            if !any_dirty {
                return glib::Propagation::Proceed;
            }
            let dlg = adw::Window::builder()
                .modal(true)
                .transient_for(&window_close_clone)
                .default_width(360)
                .resizable(false)
                .title("Unsaved Changes")
                .build();
            dlg.add_css_class("dialog-border");

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let hbar = adw::HeaderBar::new();
            hbar.set_show_end_title_buttons(false);
            vbox.append(&hbar);

            let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
            body.set_margin_top(12);
            body.set_margin_bottom(24);
            body.set_margin_start(24);
            body.set_margin_end(24);

            let heading = gtk::Label::new(Some("Discard unsaved changes?"));
            heading.add_css_class("title-3");
            heading.set_halign(gtk::Align::Start);
            body.append(&heading);

            let msg = gtk::Label::new(Some("You have unsaved changes in one or more open tabs. If you close now, those changes will be lost."));
            msg.set_wrap(true);
            msg.set_halign(gtk::Align::Start);
            msg.add_css_class("dim-label");
            body.append(&msg);

            let btn_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            btn_row.set_halign(gtk::Align::End);
            btn_row.set_margin_top(8);
            let cancel_btn = gtk::Button::with_label("Cancel");
            let discard_btn = gtk::Button::with_label("Discard All & Close");
            discard_btn.add_css_class("destructive-action");
            btn_row.append(&cancel_btn);
            btn_row.append(&discard_btn);
            body.append(&btn_row);

            vbox.append(&body);
            dlg.set_content(Some(&vbox));

            let dlg_c = dlg.clone();
            cancel_btn.connect_clicked(move |_| dlg_c.close());

            let dlg_d = dlg.clone();
            let window_inner = window_close_clone.clone();
            discard_btn.connect_clicked(move |_| {
                dlg_d.close();
                window_inner.destroy();
            });

            dlg.present();
            glib::Propagation::Stop
        });

        Self {
            window,
            state,
        }
    }

    pub fn create_new_tab(
        state: &Rc<RefCell<AppState>>,
        file: Option<PathBuf>,
    ) -> Rc<RefCell<TabState>> {
        let config = state.borrow().config.clone();
        
        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_vexpand(true);
        
        let editor_scroll = gtk::ScrolledWindow::new();
        editor_scroll.set_overlay_scrolling(false);
        
        let editor = source::View::new();
        editor.set_monospace(true);
        editor.set_show_line_numbers(config.appearance.show_line_numbers);
        editor.set_highlight_current_line(true);
        editor.set_auto_indent(true);
        editor.set_insert_spaces_instead_of_tabs(true);
        editor.set_tab_width(4);
        
        if config.appearance.word_wrap {
            editor.set_wrap_mode(gtk::WrapMode::WordChar);
        } else {
            editor.set_wrap_mode(gtk::WrapMode::None);
        }
        
        let buffer = editor.buffer().downcast::<source::Buffer>().unwrap();
        buffer.set_enable_undo(true);
        buffer.set_max_undo_levels(config.history_length as u32);
        
        let scheme_manager = source::StyleSchemeManager::default();
        let style_manager = adw::StyleManager::default();
        
        let update_scheme = {
            let buffer = buffer.clone();
            let scheme_manager = scheme_manager.clone();
            move |dark: bool| {
                let scheme_id = if dark { "adwaita-dark" } else { "adwaita" };
                if let Some(scheme) = scheme_manager.scheme(scheme_id) {
                    buffer.set_style_scheme(Some(&scheme));
                } else {
                    let fallback = if dark { "classic-dark" } else { "classic" };
                    if let Some(scheme) = scheme_manager.scheme(fallback) {
                        buffer.set_style_scheme(Some(&scheme));
                    }
                }
            }
        };
        
        update_scheme(style_manager.is_dark());
        
        let buffer_scheme_clone = buffer.clone();
        style_manager.connect_dark_notify(move |sm| {
            let scheme_id = if sm.is_dark() { "adwaita-dark" } else { "adwaita" };
            if let Some(scheme) = scheme_manager.scheme(scheme_id) {
                buffer_scheme_clone.set_style_scheme(Some(&scheme));
            }
        });
        
        editor_scroll.set_child(Some(&editor));
        paned.set_start_child(Some(&editor_scroll));
        paned.set_resize_start_child(true);
        
        let settings = webkit6::Settings::builder()
            .allow_file_access_from_file_urls(true)
            .allow_universal_access_from_file_urls(true)
            .build();
        let preview = webkit6::WebView::builder()
            .settings(&settings)
            .build();
        preview.set_vexpand(true);
        preview.set_hexpand(true);
        preview.set_visible(false); // Initially hide preview unless .smd is opened
        paned.set_end_child(Some(&preview));
        paned.set_resize_end_child(true);
        
        let local_only = config.appearance.local_only;
        preview.connect_decide_policy(move |_, decision, decision_type| {
            if decision_type == webkit6::PolicyDecisionType::NavigationAction {
                if let Ok(nav_decision) = decision.clone().downcast::<webkit6::NavigationPolicyDecision>() {
                    if let Some(action) = nav_decision.navigation_action() {
                        if action.navigation_type() == webkit6::NavigationType::LinkClicked {
                            let uri = action.request()
                                .and_then(|r| r.uri())
                                .map(|s| s.to_string())
                                .unwrap_or_default();
                            decision.ignore();
                            if !uri.is_empty() {
                                let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>);
                            }
                            return true;
                        }
                        let uri = action.request()
                            .and_then(|r| r.uri())
                            .map(|s| s.to_string())
                            .unwrap_or_default();
                        let is_external = !uri.starts_with("file://") && !uri.starts_with("about:");
                        if local_only && is_external {
                            decision.ignore();
                            return true;
                        }
                    }
                }
            }
            false
        });
        
        let tab_page = state.borrow().tab_view.append(&paned);
        
        let (title, is_smd) = if let Some(p) = &file {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("Untitled").to_string();
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            (name, ext == "smd")
        } else {
            ("Untitled".to_string(), false)
        };
        tab_page.set_title(&title);
        if let Some(p) = &file {
            tab_page.set_tooltip(&p.display().to_string());
        }
        
        let tab_state = Rc::new(RefCell::new(TabState {
            file: file.clone(),
            is_dirty: false,
            buffer: buffer.clone(),
            editor_view: editor.clone(),
            editor_scroll: editor_scroll.clone(),
            preview: preview.clone(),
            paned: paned.clone(),
            nav_history: Vec::new(),
            nav_index: 0,
            is_navigating: false,
            tab_page: tab_page.clone(),
        }));
        
        let state_changed_clone = state.clone();
        let tab_changed_clone = tab_state.clone();
        let preview_changed_clone = preview.clone();
        buffer.connect_changed(move |buf| {
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
            
            let mut tab = tab_changed_clone.borrow_mut();
            tab.is_dirty = true;
            tab.tab_page.set_needs_attention(true);
            
            let s = state_changed_clone.borrow();
            
            // Sync save button and title if this is the active tab
            if let Some(selected_page) = s.tab_view.selected_page() {
                if selected_page == tab.tab_page {
                    if let Some(btn) = &s.save_btn {
                        btn.set_sensitive(true);
                    }
                    if let Some(root) = s.tab_view.root() {
                        if let Ok(w) = root.downcast::<gtk::Window>() {
                            let title = tab.file.as_ref()
                                .map(|p| format!("SFMDE - {}*", p.display()))
                                .unwrap_or_else(|| "SFMDE - Untitled*".to_string());
                            w.set_title(Some(&title));
                        }
                    }
                }
            }
            
            let is_smd = tab.file.as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .map(|ext| ext == "smd")
                .unwrap_or(false);
            let base_uri = tab.file.as_ref()
                .and_then(|p| p.parent())
                .and_then(|d| d.to_str())
                .map(|d| format!("file://{}/", d));
                
            let config = s.config.clone();
            let highlight_color = config.appearance.highlight_color.clone();
            let local_only = config.appearance.local_only;
            let mut body = crate::parser::render_to_html(&text, &config);
            
            if !is_smd {
                body = format!(
                    r#"<p class="warning">&#9888; Preview only available for .smd files</p>{}"#,
                    body
                );
            }
            
            let css = crate::config::get_style_css_path()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .unwrap_or_default();
                
            let html = crate::parser::build_html_document(
                &body,
                &css,
                s.preview_color_scheme,
                &highlight_color,
                local_only,
            );
            preview_changed_clone.load_html(&html, base_uri.as_deref());
            
            // Sync undo/redo buttons if active
            if let Some(selected_page) = s.tab_view.selected_page() {
                if selected_page == tab.tab_page {
                    if let Some(undo_btn) = &s.undo_btn {
                        undo_btn.set_sensitive(buf.can_undo());
                    }
                    if let Some(redo_btn) = &s.redo_btn {
                        redo_btn.set_sensitive(buf.can_redo());
                    }
                }
            }
        });
        
        let state_cursor_clone = state.clone();
        let tab_cursor_clone = tab_state.clone();
        let preview_cursor_clone = preview.clone();
        buffer.connect_cursor_position_notify(move |buf| {
            let offset = buf.cursor_position();
            let iter = buf.iter_at_offset(offset);
            let line = iter.line() + 1;
            
            let s = state_cursor_clone.borrow();
            let mut tab = tab_cursor_clone.borrow_mut();
            
            if let Some(selected_page) = s.tab_view.selected_page() {
                if selected_page == tab.tab_page {
                    if let Some(label) = &s.cursor_label {
                        label.set_text(&format!("Line: {}, Col: {}", line, iter.line_offset() + 1));
                    }
                }
            }
            
            let js = format!("if(window._sfmde_setCursor)window._sfmde_setCursor({line});");
            preview_cursor_clone.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
            
            if !tab.is_navigating {
                let nav_state = NavState {
                    file: tab.file.clone(),
                    cursor_offset: offset,
                };
                
                let should_push = tab.nav_history.get(tab.nav_index).map_or(true, |last| {
                    last.file != nav_state.file || (last.cursor_offset - nav_state.cursor_offset).abs() > 100
                });
                
                if should_push {
                    let new_len = tab.nav_index + 1;
                    tab.nav_history.truncate(new_len);
                    tab.nav_history.push(nav_state);
                    if tab.nav_history.len() > s.config.history_length {
                        tab.nav_history.remove(0);
                    }
                    tab.nav_index = tab.nav_history.len().saturating_sub(1);
                }
            }
        });
        
        let preview_scroll_clone = preview.clone();
        editor_scroll.vadjustment().connect_value_changed(move |adj| {
            let upper = adj.upper() - adj.page_size();
            if upper <= 0.0 { return; }
            let fraction = (adj.value() / upper).clamp(0.0, 1.0);
            let js = format!("if(window._sfmde_syncScroll)window._sfmde_syncScroll({fraction:.4});");
            preview_scroll_clone.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
        });
        
        if let Some(p) = &file {
            if let Ok(content) = std::fs::read_to_string(p) {
                buffer.set_text(&content);
                tab_state.borrow_mut().is_dirty = false;
                tab_page.set_needs_attention(false);
            }
        }
        
        // Show preview if .smd file
        preview.set_visible(is_smd);
        
        state.borrow_mut().open_tabs.push(tab_state.clone());
        state.borrow().tab_view.set_selected_page(&tab_page);
        state.borrow().main_stack.set_visible_child_name("editor");
        
        App::sync_ui_to_active_tab(state);
        
        tab_state
    }

    pub fn save_tab_with_callback<F>(
        state: &Rc<RefCell<AppState>>,
        tab: &Rc<RefCell<TabState>>,
        parent_window: &impl IsA<gtk::Window>,
        callback: F,
    ) where
        F: Fn(bool) + 'static,
    {
        let tab_clone = tab.clone();
        let path = tab.borrow().file.clone();
        if let Some(path) = path {
            let buffer = tab.borrow().buffer.clone();
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            if std::fs::write(&path, text).is_ok() {
                tab_clone.borrow_mut().is_dirty = false;
                tab_clone.borrow().tab_page.set_needs_attention(false);
                callback(true);
            } else {
                callback(false);
            }
        } else {
            let file_dialog = gtk::FileDialog::new();
            let state_inner = state.clone();
            let tab_inner = tab.clone();
            let callback = Rc::new(callback);
            let callback_clone = callback.clone();
            file_dialog.save(Some(parent_window), gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        let buffer = tab_inner.borrow().buffer.clone();
                        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                        if std::fs::write(&path, text).is_ok() {
                            let mut t = tab_inner.borrow_mut();
                            t.file = Some(path.clone());
                            t.is_dirty = false;
                            t.tab_page.set_needs_attention(false);
                            
                            let title = path.file_name().and_then(|n| n.to_str()).unwrap_or("Untitled").to_string();
                            t.tab_page.set_title(&title);
                            t.tab_page.set_tooltip(&path.display().to_string());
                            
                            // Trigger recent files addition
                            let s = state_inner.borrow();
                            if let Some(path_str) = path.to_str() {
                                let mut s_config = s.config.clone();
                                crate::config::push_recent_file(&mut s_config, path_str);
                                let _ = crate::config::save_global_config(&s_config);
                            }
                            drop(s);
                            rebuild_recents_menu(&state_inner);
                            
                            // Trigger a dummy edit to force preview refresh with new file state
                            buffer.begin_user_action();
                            let mut start = buffer.start_iter();
                            buffer.insert(&mut start, "");
                            buffer.end_user_action();
                            t.is_dirty = false;
                            
                            callback_clone(true);
                            return;
                        }
                    }
                }
                callback_clone(false);
            });
        }
    }

    pub fn trigger_open_dialog(
        state: &Rc<RefCell<AppState>>,
        parent_window: &adw::ApplicationWindow,
    ) {
        let file_dialog = gtk::FileDialog::new();
        // Restore last opened directory
        let last_dir = state.borrow().config.appearance.last_open_dir.clone();
        if !last_dir.is_empty() {
            let gfile = gio::File::for_path(&last_dir);
            file_dialog.set_initial_folder(Some(&gfile));
        }
        
        let state_inner = state.clone();
        file_dialog.open(Some(parent_window), gio::Cancellable::NONE, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    App::open_file_in_tab(&state_inner, &path);
                }
            }
        });
    }

    pub fn open_file_in_tab(state: &Rc<RefCell<AppState>>, path: &std::path::Path) {
        {
            let s = state.borrow();
            for tab in &s.open_tabs {
                if tab.borrow().file.as_ref() == Some(&path.to_path_buf()) {
                    s.tab_view.set_selected_page(&tab.borrow().tab_page);
                    return;
                }
            }
        }
        
        let _tab = App::create_new_tab(state, Some(path.to_path_buf()));
        
        let mut s = state.borrow_mut();
        if let Some(dir) = path.parent().and_then(|d| d.to_str()) {
            s.config.appearance.last_open_dir = dir.to_string();
        }
        if let Some(path_str) = path.to_str() {
            crate::config::push_recent_file(&mut s.config, path_str);
            let _ = crate::config::save_global_config(&s.config);
        }
        drop(s);
        rebuild_recents_menu(state);
        App::sync_ui_to_active_tab(state);
    }

    pub fn sync_ui_to_active_tab(state: &Rc<RefCell<AppState>>) {
        let s = state.borrow();
        if let Some(tab) = s.get_active_tab() {
            let tab_borrow = tab.borrow();
            
            // 1. Title
            if let Some(root) = s.tab_view.root() {
                if let Ok(w) = root.downcast::<gtk::Window>() {
                    let title = if tab_borrow.is_dirty {
                        tab_borrow.file.as_ref()
                            .map(|p| format!("SFMDE - {}*", p.display()))
                            .unwrap_or_else(|| "SFMDE - Untitled*".to_string())
                    } else {
                        tab_borrow.file.as_ref()
                            .map(|p| format!("SFMDE - {}", p.display()))
                            .unwrap_or_else(|| "SFMDE - Untitled".to_string())
                    };
                    w.set_title(Some(&title));
                }
            }

            // 2. Cursor label
            if let Some(label) = &s.cursor_label {
                let offset = tab_borrow.buffer.cursor_position();
                let iter = tab_borrow.buffer.iter_at_offset(offset);
                let line = iter.line() + 1;
                label.set_text(&format!("Line: {}, Col: {}", line, iter.line_offset() + 1));
                label.set_visible(s.config.appearance.show_line_col);
            }

            // 3. Save button sensitivity
            if let Some(btn) = &s.save_btn {
                btn.set_sensitive(true);
            }

            // 4. Undo/Redo sensitivities
            if let Some(undo_btn) = &s.undo_btn {
                undo_btn.set_sensitive(tab_borrow.buffer.can_undo());
            }
            if let Some(redo_btn) = &s.redo_btn {
                redo_btn.set_sensitive(tab_borrow.buffer.can_redo());
            }

            // 5. Preview/Editor toggle states
            if let Some(toggle) = &s.preview_toggle {
                toggle.set_active(tab_borrow.preview.is_visible());
            }
            if let Some(toggle) = &s.editor_toggle {
                toggle.set_active(tab_borrow.editor_scroll.is_visible());
            }

            // 6. Refresh formatting toolbar
            drop(s);
            refresh_toolbar(state.clone(), None);
        } else {
            // No active tabs: show empty screen and disable global buttons
            if let Some(root) = s.tab_view.root() {
                if let Ok(w) = root.downcast::<gtk::Window>() {
                    w.set_title(Some("SFMDE"));
                }
            }

            if let Some(label) = &s.cursor_label {
                label.set_text("");
                label.set_visible(false);
            }

            if let Some(btn) = &s.save_btn {
                btn.set_sensitive(false);
            }
            if let Some(undo_btn) = &s.undo_btn {
                undo_btn.set_sensitive(false);
            }
            if let Some(redo_btn) = &s.redo_btn {
                redo_btn.set_sensitive(false);
            }
            
            s.main_stack.set_visible_child_name("empty");
            
            drop(s);
            refresh_toolbar(state.clone(), None);
        }
    }

    pub fn open_file(&self, path: &std::path::Path) {
        App::open_file_in_tab(&self.state, path);
    }
}
