use gtk4 as gtk;
use gtk::gio;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use crate::ui::{AppState, TabState, App};

pub(crate) fn rebuild_recents_menu(state: &Rc<RefCell<AppState>>) {
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

impl App {
    pub fn show_message_dialog(
        parent_window: &impl IsA<gtk::Window>,
        title: &str,
        heading: &str,
        message: &str,
    ) {
        let dlg = adw::Window::builder()
            .modal(true)
            .transient_for(parent_window)
            .default_width(320)
            .resizable(false)
            .title(title)
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

        let head = gtk::Label::new(Some(heading));
        head.add_css_class("title-3");
        head.set_halign(gtk::Align::Start);
        body.append(&head);

        let msg = gtk::Label::new(Some(message));
        msg.set_wrap(true);
        msg.set_halign(gtk::Align::Start);
        msg.add_css_class("dim-label");
        body.append(&msg);

        let btn_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        btn_row.set_halign(gtk::Align::End);
        btn_row.set_margin_top(8);
        let ok_btn = gtk::Button::with_label("OK");
        ok_btn.add_css_class("suggested-action");
        btn_row.append(&ok_btn);
        body.append(&btn_row);

        vbox.append(&body);
        dlg.set_content(Some(&vbox));

        let dlg_c = dlg.clone();
        ok_btn.connect_clicked(move |_| dlg_c.close());

        dlg.present();
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

    pub fn export_tab_as_html(
        state: &Rc<RefCell<AppState>>,
        tab: &Rc<RefCell<TabState>>,
        parent_window: &impl IsA<gtk::Window>,
    ) {
        let file_dialog = gtk::FileDialog::new();
        
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("HTML Files"));
        let _ = filter.add_mime_type("text/html");
        filter.add_suffix("html");
        filter.add_suffix("htm");

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        file_dialog.set_filters(Some(&filters));

        let last_dir = state.borrow().config.appearance.last_open_dir.clone();
        if !last_dir.is_empty() {
            let gfile = gio::File::for_path(&last_dir);
            file_dialog.set_initial_folder(Some(&gfile));
        }

        let tab_borrow = tab.borrow();
        let default_name = if let Some(ref path) = tab_borrow.file {
            path.with_extension("html")
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("document.html")
                .to_string()
        } else {
            "document.html".to_string()
        };
        file_dialog.set_initial_name(Some(&default_name));
        drop(tab_borrow);

        let state_inner = state.clone();
        let tab_inner = tab.clone();
        let parent_window_clone = parent_window.clone();

        file_dialog.save(Some(parent_window), gio::Cancellable::NONE, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    let buffer = tab_inner.borrow().buffer.clone();
                    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                    let config = state_inner.borrow().config.clone();
                    
                    let body_html = crate::parser::render_to_html_inline(&text, &config);
                    
                    let title = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Exported Document");
                    
                    let full_html = crate::parser::build_html_document_inline_styles(&body_html, title);
                    
                    if std::fs::write(&path, full_html).is_ok() {
                        if let Some(parent) = path.parent() {
                            let mut s = state_inner.borrow_mut();
                            s.config.appearance.last_open_dir = parent.to_string_lossy().to_string();
                            let _ = crate::config::save_global_config(&s.config);
                        }
                    } else {
                        App::show_message_dialog(
                            &parent_window_clone,
                            "Export Error",
                            "Failed to Export HTML",
                            &format!("Could not write to file: {}", path.display())
                        );
                    }
                }
            }
        });
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

    pub fn open_file_in_tab(state: &Rc<RefCell<AppState>>, path: &Path) {
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

    pub fn open_file(&self, path: &Path) {
        App::open_file_in_tab(&self.state, path);
    }
}
