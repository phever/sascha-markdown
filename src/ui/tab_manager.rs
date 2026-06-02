use gtk4 as gtk;
use gtk::gio;
use gtk::prelude::*;
use libadwaita as adw;
use sourceview5 as source;
use source::prelude::*;
use webkit6::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use crate::ui::{AppState, TabState, NavState, App};
use crate::ui::toolbar::refresh_toolbar;


impl App {
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
}
