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
            is_dirty: false,
            toolbar: None,
            buffer: None,
            editor_view: None,
            cursor_label: None,
            preview_toggle: None,
            editor_visible: true,
            preview_visible: false,
            preview_color_scheme: 0,
            css_provider,
            recents_menu: None,
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

        // Undo/Redo (left side, where nav was)
        let undo_redo_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        undo_redo_box.add_css_class("linked");
        header.pack_start(&undo_redo_box);

        let undo_btn = gtk::Button::from_icon_name("edit-undo-symbolic");
        undo_btn.set_tooltip_text(Some("Undo"));
        undo_redo_box.append(&undo_btn);

        let redo_btn = gtk::Button::from_icon_name("edit-redo-symbolic");
        redo_btn.set_tooltip_text(Some("Redo"));
        undo_redo_box.append(&redo_btn);

        // File buttons
        let new_btn = gtk::Button::from_icon_name("document-new-symbolic");
        new_btn.set_tooltip_text(Some("New File"));
        header.pack_start(&new_btn);

        let open_btn = gtk::Button::with_label("Open");
        header.pack_start(&open_btn);

        let save_btn = gtk::Button::with_label("Save");
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
        state.borrow_mut().preview_toggle = Some(toggle_preview_btn.clone());

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

        // Populate recents_menu from current config and store in state
        {
            let recent = config.recent_files.clone();
            for path in recent.iter().take(3) {
                let label = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path);
                let item = gio::MenuItem::new(Some(label), None);
                item.set_action_and_target_value(
                    Some("app.open-recent"),
                    Some(&path.to_variant()),
                );
                recents_menu.append_item(&item);
            }
        }
        state.borrow_mut().recents_menu = Some(recents_menu);

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
        // Disable overlay scrolling so the horizontal scrollbar never floats
        // over the last line of text.
        editor_scroll.set_overlay_scrolling(false);
        let editor = source::View::new();
        editor.set_monospace(true);
        editor.set_show_line_numbers(config.appearance.show_line_numbers);
        editor.set_highlight_current_line(true);
        editor.set_auto_indent(true);
        editor.set_insert_spaces_instead_of_tabs(true);
        editor.set_tab_width(4);
        
        let buffer = editor.buffer().downcast::<source::Buffer>().unwrap();
        buffer.set_enable_undo(true);
        buffer.set_max_undo_levels(state.borrow().config.history_length as u32);
        state.borrow_mut().buffer = Some(buffer.clone());
        state.borrow_mut().editor_view = Some(editor.clone());

        if state.borrow().config.appearance.word_wrap {
            editor.set_wrap_mode(gtk::WrapMode::WordChar);
        } else {
            editor.set_wrap_mode(gtk::WrapMode::None);
        }

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

        // Reload appearance CSS (includes .dialog-border) now that state is built
        apply_appearance(&state.borrow().css_provider, &config.appearance);
        
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

        // Block external (non-file://) navigation when local_only is enabled;
        // always open clicked links in the system browser rather than the preview pane.
        let local_only_init = config.appearance.local_only;
        preview.connect_decide_policy(move |_, decision, decision_type| {
            if decision_type == PolicyDecisionType::NavigationAction {
                if let Ok(nav_decision) = decision.clone().downcast::<NavigationPolicyDecision>() {
                    if let Some(action) = nav_decision.navigation_action() {
                        if action.navigation_type() == NavigationType::LinkClicked {
                            // Never navigate the preview pane — open all links externally.
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
                        // Block non-file:// navigation (e.g. form submits, redirects) when local_only
                        let uri = action.request()
                            .and_then(|r| r.uri())
                            .map(|s| s.to_string())
                            .unwrap_or_default();
                        let is_external = !uri.starts_with("file://") && !uri.starts_with("about:");
                        if local_only_init && is_external {
                            decision.ignore();
                            return true;
                        }
                    }
                }
            }
            false
        });

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

            state_clone.borrow_mut().is_dirty = true;

            let (is_smd, config, base_uri, preview_color_scheme, _) = {
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
                (is_smd, s.config.clone(), base_uri, s.preview_color_scheme, s.config.appearance.local_only)
            };

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

            let html = crate::parser::build_html_document(&body, &css, preview_color_scheme, &highlight_color, local_only);
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
        cursor_label.set_visible(state.borrow().config.appearance.show_line_col);
        status_bar.append(&cursor_label);
        state.borrow_mut().cursor_label = Some(cursor_label.clone());

        let buffer_state_clone = state.clone();
        let preview_cursor_clone = preview.clone();
        buffer.connect_cursor_position_notify(move |buf| {
            let offset = buf.cursor_position();
            let iter = buf.iter_at_offset(offset);
            let line = iter.line() + 1;
            cursor_label.set_text(&format!("Line: {}, Col: {}", line, iter.line_offset() + 1));

            // Update cursor indicator in preview
            let js = format!("if(window._sfmde_setCursor)window._sfmde_setCursor({line});");
            preview_cursor_clone.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});

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

        // Editor scroll → preview scroll sync
        let preview_scroll_clone = preview.clone();
        editor_scroll.vadjustment().connect_value_changed(move |adj| {
            let upper = adj.upper() - adj.page_size();
            if upper <= 0.0 { return; }
            let fraction = (adj.value() / upper).clamp(0.0, 1.0);
            let js = format!("if(window._sfmde_syncScroll)window._sfmde_syncScroll({fraction:.4});");
            preview_scroll_clone.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
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
            // Restore last opened directory
            let last_dir = state_f_clone.borrow().config.appearance.last_open_dir.clone();
            if !last_dir.is_empty() {
                let gfile = gio::File::for_path(&last_dir);
                file_dialog.set_initial_folder(Some(&gfile));
            }
            let window_inner = window_clone.clone();
            let buffer = buffer_clone.clone();
            let state = state_f_clone.clone();
            let toggle_p = toggle_p_f_clone.clone();
            file_dialog.open(Some(&window_clone), gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            {
                                let mut s = state.borrow_mut();
                                if let Some(dir) = path.parent().and_then(|d| d.to_str()) {
                                    s.config.appearance.last_open_dir = dir.to_string();
                                }
                                s.current_file = Some(path.clone());
                                s.is_dirty = false;
                                if let Some(path_str) = path.to_str() {
                                    crate::config::push_recent_file(&mut s.config, path_str);
                                    let _ = crate::config::save_global_config(&s.config);
                                }
                            }
                            rebuild_recents_menu(&state);
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
                            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                            if std::fs::write(&path, text).is_ok() {
                                let mut s = state.borrow_mut();
                                s.current_file = Some(path.clone());
                                s.is_dirty = false;
                            }
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
                            // connect_changed fires synchronously above and marks dirty; undo that
                            state.borrow_mut().is_dirty = false;
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
                if std::fs::write(path, text).is_ok() {
                    state_s_clone.borrow_mut().is_dirty = false;
                }
            } else {
                save_as_clone();
            }
        });

        // Local-Only toggle action
        let state_lo_clone = state.clone();
        let buffer_lo_clone = buffer.clone();
        local_only_btn.connect_toggled(move |btn| {
            let active = btn.is_active();
            {
                let mut s = state_lo_clone.borrow_mut();
                s.config.appearance.local_only = active;
                let _ = crate::config::save_global_config(&s.config);
            }
            // Trigger preview re-render so CSP tag is added/removed
            use glib::prelude::*;
            buffer_lo_clone.emit_by_name::<()>("changed", &[]);
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
            if let Some(view) = &s.editor_view {
                view.set_wrap_mode(if new_val {
                    gtk4::WrapMode::WordChar
                } else {
                    gtk4::WrapMode::None
                });
            }
        });
        app.add_action(&action_word_wrap);

        // Open Recent action — parameter is the file path string
        let action_open_recent = gio::SimpleAction::new(
            "open-recent",
            Some(&String::static_variant_type()),
        );
        let state_or_clone = state.clone();
        let buffer_or_clone = buffer.clone();
        let window_or_clone = window.clone();
        let toggle_or_clone = toggle_preview_btn.clone();
        action_open_recent.connect_activate(move |_, param| {
            if let Some(path_str) = param.and_then(|v| v.get::<String>()) {
                let path = std::path::PathBuf::from(&path_str);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let is_smd = path.extension().and_then(|e| e.to_str()).map(|e| e == "smd").unwrap_or(false);
                    {
                        let mut s = state_or_clone.borrow_mut();
                        s.current_file = Some(path.clone());
                        s.is_dirty = false;
                        if let Some(dir) = path.parent().and_then(|d| d.to_str()) {
                            s.config.appearance.last_open_dir = dir.to_string();
                        }
                    }
                    buffer_or_clone.set_text(&content);
                    window_or_clone.set_title(Some(&format!("SFMDE - {}", path.display())));
                    toggle_or_clone.set_active(is_smd);
                }
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
            if !state_close_clone.borrow().is_dirty {
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

            let msg = gtk::Label::new(Some("Your changes will be lost if you close without saving."));
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
            editor,
            preview,
            state,
        }
    }

    /// Open a file by path, replacing the current buffer contents.
    pub fn open_file(&self, path: &std::path::Path) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let is_smd = path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s == "smd")
                .unwrap_or(false);
            {
                let mut s = self.state.borrow_mut();
                s.current_file = Some(path.to_path_buf());
                s.is_dirty = false;
                if let Some(dir) = path.parent().and_then(|d| d.to_str()) {
                    s.config.appearance.last_open_dir = dir.to_string();
                }
                if let Some(path_str) = path.to_str() {
                    crate::config::push_recent_file(&mut s.config, path_str);
                    let _ = crate::config::save_global_config(&s.config);
                }
                if let Some(toggle) = &s.preview_toggle {
                    toggle.set_active(is_smd);
                }
            }
            rebuild_recents_menu(&self.state);
            let buffer = self.editor.buffer()
                .downcast::<source::Buffer>()
                .unwrap();
            buffer.set_text(&content);
            self.window.set_title(Some(&format!("SFMDE - {}", path.display())));
        }
    }
}
