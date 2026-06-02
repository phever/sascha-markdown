use gtk4 as gtk;
use gtk::glib;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use sourceview5 as source;
use source::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ui::{AppState, App};

pub fn connect_tab_events(state: &Rc<RefCell<AppState>>, tab_view: &adw::TabView) {
    let state_notify_clone = state.clone();
    tab_view.connect_notify_local(Some("selected-page"), move |_, _| {
        App::sync_ui_to_active_tab(&state_notify_clone);
    });

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
}

pub fn connect_ui_signals(
    state: &Rc<RefCell<AppState>>,
    window: &adw::ApplicationWindow,
    new_btn: &gtk::Button,
    open_btn: &gtk::Button,
    save_btn: &gtk::Button,
    undo_btn: &gtk::Button,
    redo_btn: &gtk::Button,
    welcome_new_btn: &gtk::Button,
    welcome_open_btn: &gtk::Button,
    toggle_editor_btn: &gtk::ToggleButton,
    toggle_preview_btn: &gtk::ToggleButton,
    preview_mode_btn: &gtk::Button,
    local_only_btn: &gtk::ToggleButton,
) {
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
}
