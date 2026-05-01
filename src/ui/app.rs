use gtk4 as gtk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use sourceview5 as source;
use source::prelude::*;
use webkit6::prelude::*;
use webkit6::{NavigationPolicyDecision, NavigationType, PolicyDecisionType, WebView};
use crate::config::Config;
use std::cell::RefCell;
use std::rc::Rc;
use crate::ui::{AppState, NavState, App};
use crate::ui::markup::apply_markup;
use crate::ui::toolbar::refresh_toolbar;
use crate::ui::settings::show_settings_dialog;

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

        let state = Rc::new(RefCell::new(AppState {
            current_file: None,
            config: config.clone(),
            nav_history: Vec::new(),
            nav_index: 0,
            is_navigating: false,
            toolbar: None,
            buffer: None,
            editor_visible: true,
            preview_visible: false,
            preview_color_scheme: 0,
            css_provider,
        }));

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

        // Navigation
        let nav_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        nav_box.add_css_class("linked");
        header.pack_start(&nav_box);

        let back_btn = gtk::Button::from_icon_name("go-previous-symbolic");
        back_btn.set_tooltip_text(Some("Back"));
        nav_box.append(&back_btn);

        let forward_btn = gtk::Button::from_icon_name("go-next-symbolic");
        forward_btn.set_tooltip_text(Some("Forward"));
        nav_box.append(&forward_btn);

        // File buttons
        let new_btn = gtk::Button::from_icon_name("document-new-symbolic");
        new_btn.set_tooltip_text(Some("New File"));
        header.pack_start(&new_btn);

        let open_btn = gtk::Button::with_label("Open");
        header.pack_start(&open_btn);

        let save_btn = gtk::Button::with_label("Save");
        header.pack_start(&save_btn);

        // Undo/Redo
        let undo_redo_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        undo_redo_box.add_css_class("linked");
        header.pack_end(&undo_redo_box);

        let undo_btn = gtk::Button::from_icon_name("edit-undo-symbolic");
        undo_btn.set_tooltip_text(Some("Undo"));
        undo_redo_box.append(&undo_btn);

        let redo_btn = gtk::Button::from_icon_name("edit-redo-symbolic");
        redo_btn.set_tooltip_text(Some("Redo"));
        undo_redo_box.append(&redo_btn);

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
        state.borrow_mut().toolbar = Some(toolbar.clone());

        // Adaptive overflow polling (safe for non-Send types)
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

        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_vexpand(true);
        main_box.append(&paned);

        // Editor
        let editor_scroll = gtk::ScrolledWindow::new();
        let editor = source::View::new();
        editor.set_monospace(true);
        editor.set_show_line_numbers(true);
        editor.set_highlight_current_line(true);
        editor.set_auto_indent(true);
        editor.set_insert_spaces_instead_of_tabs(true);
        editor.set_tab_width(4);
        
        let buffer = editor.buffer().downcast::<source::Buffer>().unwrap();
        buffer.set_enable_undo(true);
        buffer.set_max_undo_levels(state.borrow().config.history_length as u32);
        state.borrow_mut().buffer = Some(buffer.clone());

        // Sync color scheme with system
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

        // Load custom CSS
        let css = if let Some(css_path) = crate::config::get_style_css_path() {
            std::fs::read_to_string(css_path).unwrap_or_default()
        } else {
            String::new()
        };
        let final_css = format!("{}\n.dialog-border {{ border: 2px solid @accent_color; border-radius: 10px; }}", css);
        state.borrow().css_provider.load_from_data(&final_css);
        
        update_scheme(style_manager.is_dark());
        style_manager.connect_dark_notify(move |sm| {
            update_scheme(sm.is_dark());
        });

        editor_scroll.set_child(Some(&editor));
        paned.set_start_child(Some(&editor_scroll));
        paned.set_resize_start_child(true);

        // Preview — WebView replaces the old gtk::Label
        let settings = webkit6::Settings::builder()
            .allow_file_access_from_file_urls(true)
            .allow_universal_access_from_file_urls(true)
            .build();
        let preview = WebView::builder()
            .settings(&settings)
            .build();
        preview.set_vexpand(true);
        preview.set_hexpand(true);
        preview.set_visible(false);
        paned.set_end_child(Some(&preview));
        paned.set_resize_end_child(true);

        let buffer_scheme_clone = buffer.clone();
        let update_scheme = {
            let buffer = buffer.clone();
            let scheme_manager = scheme_manager.clone();
            let buffer_ref = buffer_scheme_clone.clone();
            move |dark: bool| {
                // Update Editor
                let scheme_id = if dark { "adwaita-dark" } else { "adwaita" };
                if let Some(scheme) = scheme_manager.scheme(scheme_id) {
                    buffer.set_style_scheme(Some(&scheme));
                } else {
                    let fallback = if dark { "classic-dark" } else { "classic" };
                    if let Some(scheme) = scheme_manager.scheme(fallback) {
                        buffer.set_style_scheme(Some(&scheme));
                    }
                }

                // Update Preview (WebView)
                // We emit a changed signal to the buffer to trigger a re-render of the HTML with the new theme
                use glib::prelude::*;
                buffer_ref.emit_by_name::<()>("changed", &[]);
            }
        };

        let preview_mode_btn_clone = preview_mode_btn.clone();
        let state_mode_clone = state.clone();
        let style_manager_mode_clone = style_manager.clone();
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

            // Trigger refresh
            update_scheme_mode_clone(style_manager_mode_clone.is_dark());
        });

        // Open links in the system browser instead of navigating the WebView.
        let window_clone_link = window.clone();
        preview.connect_decide_policy(move |_, decision, decision_type| {
            if decision_type == PolicyDecisionType::NavigationAction {
                if let Ok(nav) = decision.clone().downcast::<NavigationPolicyDecision>() {
                    if let Some(action) = nav.navigation_action() {
                        if action.navigation_type() == NavigationType::LinkClicked {
                            if let Some(uri) = action.request().and_then(|r| r.uri()) {
                                let launcher = gtk::UriLauncher::new(&uri);
                                launcher.launch(Some(&window_clone_link), None::<&gio::Cancellable>, |_| {});
                            }
                            nav.ignore();
                            return true;
                        }
                    }
                }
            }
            false
        });

        let editor_scroll_clone = editor_scroll.clone();
        let preview_clone_toggle = preview.clone();
        let state_toggle_clone = state.clone();
        
        toggle_editor_btn.connect_toggled(move |btn| {
            let visible = btn.is_active();
            let should_revert = {
                let mut s = state_toggle_clone.borrow_mut();
                if !visible && !s.preview_visible {
                    true
                } else {
                    s.editor_visible = visible;
                    false
                }
            };

            if should_revert {
                btn.set_active(true);
            } else {
                editor_scroll_clone.set_visible(visible);
            }
        });

        let state_toggle_p_clone = state.clone();
        let preview_clone_toggle2 = preview_clone_toggle.clone();
        let paned_clone_p = paned.clone();
        toggle_preview_btn.connect_toggled(move |btn| {
            let visible = btn.is_active();
            let should_revert = {
                let mut s = state_toggle_p_clone.borrow_mut();
                if !visible && !s.editor_visible {
                    true
                } else {
                    s.preview_visible = visible;
                    false
                }
            };

            if should_revert {
                btn.set_active(true);
            } else {
                preview_clone_toggle2.set_visible(visible);
                if visible {
                    let pos = paned_clone_p.position();
                    let width = paned_clone_p.width();
                    if pos <= 0 || (width > 0 && pos >= width - 50) {
                        paned_clone_p.set_position(width / 2);
                        if width == 0 {
                            paned_clone_p.set_position(500);
                        }
                    }
                }
            }
        });

        let preview_clone = preview.clone();
        let state_clone = state.clone();
        buffer.connect_changed(move |buf| {
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);

            let (is_smd, config, base_uri, preview_color_scheme) = {
                let s = state_clone.borrow();
                let is_smd = s.current_file.as_ref()
                    .and_then(|p| p.extension())
                    .and_then(|e| e.to_str())
                    .map(|ext| ext == "smd")
                    .unwrap_or(false);
                let base_uri = s.current_file.as_ref()
                    .and_then(|p| p.parent())
                    .and_then(|d| d.to_str())
                    .map(|d| format!("file://{}/", d));
                (is_smd, s.config.clone(), base_uri, s.preview_color_scheme)
            };

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

            let html = crate::parser::build_html_document(&body, &css, preview_color_scheme);
            preview_clone.load_html(&html, base_uri.as_deref());
        });

        // Initial toolbar population
        refresh_toolbar(state.clone(), None);

        // Status Bar
        let status_bar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        status_bar.set_margin_start(10);
        status_bar.set_margin_end(10);
        status_bar.set_margin_top(5);
        status_bar.set_margin_bottom(5);
        main_box.append(&status_bar);

        let cursor_label = gtk::Label::new(Some("Line: 1, Col: 1"));
        status_bar.append(&cursor_label);

        let buffer_state_clone = state.clone();
        buffer.connect_cursor_position_notify(move |buf| {
            let offset = buf.cursor_position();
            let iter = buf.iter_at_offset(offset);
            cursor_label.set_text(&format!("Line: {}, Col: {}", iter.line() + 1, iter.line_offset() + 1));

            // Record navigation history
            let mut s = buffer_state_clone.borrow_mut();
            if !s.is_navigating {
                let nav_state = NavState {
                    file: s.current_file.clone(),
                    cursor_offset: offset,
                };
                
                let should_push = s.nav_history.get(s.nav_index).map_or(true, |last| {
                    last.file != nav_state.file || (last.cursor_offset - nav_state.cursor_offset).abs() > 100
                });

                if should_push {
                    let new_len = s.nav_index + 1;
                    s.nav_history.truncate(new_len);
                    s.nav_history.push(nav_state);
                    if s.nav_history.len() > s.config.history_length {
                        s.nav_history.remove(0);
                    }
                    s.nav_index = s.nav_history.len().saturating_sub(1);
                }
            }
        });

        // Setup File Callbacks
        let buffer_n_clone = buffer.clone();
        let state_n_clone = state.clone();
        let window_n_clone = window.clone();
        let toggle_p_n_clone = toggle_preview_btn.clone();
        new_btn.connect_clicked(move |_| {
            state_n_clone.borrow_mut().current_file = None;
            buffer_n_clone.set_text("");
            window_n_clone.set_title(Some("SFMDE - New File"));
            toggle_p_n_clone.set_active(false);
        });

        let window_clone = window.clone();
        let buffer_clone = buffer.clone();
        let state_f_clone = state.clone();
        let toggle_p_f_clone = toggle_preview_btn.clone();
        open_btn.connect_clicked(move |_| {
            let file_dialog = gtk::FileDialog::new();
            let window_inner = window_clone.clone();
            let buffer = buffer_clone.clone();
            let state = state_f_clone.clone();
            let toggle_p = toggle_p_f_clone.clone();
            file_dialog.open(Some(&window_clone), gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            state.borrow_mut().current_file = Some(path.clone());
                            buffer.set_text(&content);
                            window_inner.set_title(Some(&format!("SFMDE - {}", path.display())));
                            
                            let is_smd = path.extension()
                                .and_then(|e| e.to_str())
                                .map(|s| s == "smd")
                                .unwrap_or(false);
                            toggle_p.set_active(is_smd);
                        }
                    }
                }
            });
        });

        let save_as = {
            let window = window.clone();
            let buffer = buffer.clone();
            let state = state.clone();
            let toggle_p = toggle_preview_btn.clone();
            move || {
                let file_dialog = gtk::FileDialog::new();
                let window_inner = window.clone();
                let buffer = buffer.clone();
                let state = state.clone();
                let toggle_p = toggle_p.clone();
                file_dialog.save(Some(&window), gio::Cancellable::NONE, move |res| {
                    if let Ok(file) = res {
                        if let Some(path) = file.path() {
                            state.borrow_mut().current_file = Some(path.clone());
                            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                            let _ = std::fs::write(&path, text);
                            window_inner.set_title(Some(&format!("SFMDE - {}", path.display())));
                            
                            let is_smd = path.extension()
                                .and_then(|e| e.to_str())
                                .map(|s| s == "smd")
                                .unwrap_or(false);
                            toggle_p.set_active(is_smd);
                            
                            // Trigger a dummy edit to force preview refresh with new file state
                            buffer.begin_user_action();
                            let mut start = buffer.start_iter();
                            buffer.insert(&mut start, "");
                            buffer.end_user_action();
                        }
                    }
                });
            }
        };

        let save_as_clone = save_as.clone();
        let state_s_clone = state.clone();
        let buffer_s_clone = buffer.clone();
        save_btn.connect_clicked(move |_| {
            let path = state_s_clone.borrow().current_file.clone();
            if let Some(path) = path {
                let text = buffer_s_clone.text(&buffer_s_clone.start_iter(), &buffer_s_clone.end_iter(), false);
                let _ = std::fs::write(path, text);
            } else {
                save_as_clone();
            }
        });

        // Navigation actions
        let state_b_clone = state.clone();
        let buffer_b_clone = buffer.clone();
        let window_b_clone = window.clone();
        let toggle_p_b_clone = toggle_preview_btn.clone();
        back_btn.connect_clicked(move |_| {
            let mut s = state_b_clone.borrow_mut();
            if s.nav_index > 0 {
                s.nav_index -= 1;
                s.is_navigating = true;
                let nav = s.nav_history[s.nav_index].clone();
                
                if nav.file != s.current_file {
                    if let Some(path) = &nav.file {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            s.current_file = Some(path.clone());
                            buffer_b_clone.set_text(&content);
                            window_b_clone.set_title(Some(&format!("SFMDE - {}", path.display())));
                            
                            let is_smd = path.extension()
                                .and_then(|e| e.to_str())
                                .map(|e| e == "smd")
                                .unwrap_or(false);
                            toggle_p_b_clone.set_active(is_smd);
                        }
                    } else {
                        s.current_file = None;
                        buffer_b_clone.set_text("");
                        window_b_clone.set_title(Some("SFMDE - New File"));
                        toggle_p_b_clone.set_active(false);
                    }
                }
                buffer_b_clone.place_cursor(&buffer_b_clone.iter_at_offset(nav.cursor_offset));
                s.is_navigating = false;
            }
        });

        let state_f_clone = state.clone();
        let buffer_f_clone = buffer.clone();
        let window_f_clone = window.clone();
        let toggle_p_f_clone = toggle_preview_btn.clone();
        forward_btn.connect_clicked(move |_| {
            let mut s = state_f_clone.borrow_mut();
            if s.nav_index + 1 < s.nav_history.len() {
                s.nav_index += 1;
                s.is_navigating = true;
                let nav = s.nav_history[s.nav_index].clone();
                
                if nav.file != s.current_file {
                    if let Some(path) = &nav.file {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            s.current_file = Some(path.clone());
                            buffer_f_clone.set_text(&content);
                            window_f_clone.set_title(Some(&format!("SFMDE - {}", path.display())));
                            
                            let is_smd = path.extension()
                                .and_then(|e| e.to_str())
                                .map(|e| e == "smd")
                                .unwrap_or(false);
                            toggle_p_f_clone.set_active(is_smd);
                        }
                    } else {
                        s.current_file = None;
                        buffer_f_clone.set_text("");
                        window_f_clone.set_title(Some("SFMDE - New File"));
                        toggle_p_f_clone.set_active(false);
                    }
                }
                buffer_f_clone.place_cursor(&buffer_f_clone.iter_at_offset(nav.cursor_offset));
                s.is_navigating = false;
            }
        });

        // Undo/Redo actions
        let buffer_u_clone = buffer.clone();
        undo_btn.connect_clicked(move |_| {
            if buffer_u_clone.can_undo() {
                buffer_u_clone.undo();
            }
        });

        let buffer_r_clone = buffer.clone();
        redo_btn.connect_clicked(move |_| {
            if buffer_r_clone.can_redo() {
                buffer_r_clone.redo();
            }
        });

        // Register Actions
        let action_new = gio::SimpleAction::new("new", None);
        let buffer_an_clone = buffer.clone();
        let state_an_clone = state.clone();
        let window_an_clone = window.clone();
        let toggle_p_an_clone = toggle_preview_btn.clone();
        action_new.connect_activate(move |_, _| {
            buffer_an_clone.set_text("");
            state_an_clone.borrow_mut().current_file = None;
            window_an_clone.set_title(Some("SFMDE - New File"));
            toggle_p_an_clone.set_active(false);
        });
        app.add_action(&action_new);

        let action_open = gio::SimpleAction::new("open", None);
        let open_btn_clone = open_btn.clone();
        action_open.connect_activate(move |_, _| {
            open_btn_clone.emit_clicked();
        });
        app.add_action(&action_open);

        let action_save = gio::SimpleAction::new("save", None);
        let save_btn_clone = save_btn.clone();
        action_save.connect_activate(move |_, _| {
            save_btn_clone.emit_clicked();
        });
        app.add_action(&action_save);

        let action_save_as = gio::SimpleAction::new("save-as", None);
        let save_as_action_clone = save_as.clone();
        action_save_as.connect_activate(move |_, _| {
            save_as_action_clone();
        });
        app.add_action(&action_save_as);

        let action_undo = gio::SimpleAction::new("undo", None);
        let undo_btn_clone = undo_btn.clone();
        action_undo.connect_activate(move |_, _| {
            undo_btn_clone.emit_clicked();
        });
        app.add_action(&action_undo);

        let action_redo = gio::SimpleAction::new("redo", None);
        let redo_btn_clone = redo_btn.clone();
        action_redo.connect_activate(move |_, _| {
            redo_btn_clone.emit_clicked();
        });
        app.add_action(&action_redo);

        let action_bold = gio::SimpleAction::new("bold", None);
        let buffer_b_clone = buffer.clone();
        let state_b_clone = state.clone();
        action_bold.connect_activate(move |_, _| {
            let symbol = state_b_clone.borrow().config.formatters.bold.symbol.clone();
            apply_markup(&buffer_b_clone, &symbol);
        });
        app.add_action(&action_bold);

        let action_italics = gio::SimpleAction::new("italics", None);
        let buffer_i_clone = buffer.clone();
        let state_i_clone = state.clone();
        action_italics.connect_activate(move |_, _| {
            let symbol = state_i_clone.borrow().config.formatters.italics.symbol.clone();
            apply_markup(&buffer_i_clone, &symbol);
        });
        app.add_action(&action_italics);

        let action_underscore = gio::SimpleAction::new("underscore", None);
        let buffer_u_clone = buffer.clone();
        let state_u_clone = state.clone();
        action_underscore.connect_activate(move |_, _| {
            let symbol = state_u_clone.borrow().config.formatters.underscore.symbol.clone();
            apply_markup(&buffer_u_clone, &symbol);
        });
        app.add_action(&action_underscore);

        let action_strikethrough = gio::SimpleAction::new("strikethrough", None);
        let buffer_s_clone = buffer.clone();
        let state_s_clone = state.clone();
        action_strikethrough.connect_activate(move |_, _| {
            let symbol = state_s_clone.borrow().config.formatters.strikethrough.symbol.clone();
            apply_markup(&buffer_s_clone, &symbol);
        });
        app.add_action(&action_strikethrough);

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
            about.present(Some(&window_ab_clone));
        });
        app.add_action(&action_about);

        let state_settings_clone = state.clone();
        let window_settings_clone = window.clone();
        settings_btn.connect_clicked(move |_| {
            show_settings_dialog(&window_settings_clone, state_settings_clone.clone());
        });

        Self {
            window,
            editor,
            preview,
            state,
        }
    }
}
