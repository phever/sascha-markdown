use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use gtk::gio;
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::config::Config;
use crate::ui::{AppState, App};
use crate::ui::toolbar::refresh_toolbar;
use crate::ui::signals::{connect_tab_events, connect_ui_signals};
use crate::ui::actions::{setup_accels, register_actions};
use crate::ui::file_ops::rebuild_recents_menu;

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
        menu_model.append(Some("Export as HTML..."), Some("app.export-html"));

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

        // Connect signals
        connect_tab_events(&state, &tab_view);
        connect_ui_signals(
            &state,
            &window,
            &new_btn,
            &open_btn,
            &save_btn,
            &undo_btn,
            &redo_btn,
            &welcome_new_btn,
            &welcome_open_btn,
            &toggle_editor_btn,
            &toggle_preview_btn,
            &preview_mode_btn,
            &local_only_btn,
        );

        // Register actions
        register_actions(app, &state, &window, &settings_btn);

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
}
