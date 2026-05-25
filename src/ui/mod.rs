pub mod app;
pub mod markup;
pub mod toolbar;
pub mod settings;

use sourceview5 as source;
use std::path::PathBuf;
use crate::config::Config;
use libadwaita as adw;
use gtk4 as gtk;
use std::rc::Rc;
use std::cell::RefCell;
use gtk4::gio;

#[derive(Clone)]
pub struct NavState {
    pub file: Option<PathBuf>,
    pub cursor_offset: i32,
}

pub struct TabState {
    pub file: Option<PathBuf>,
    pub is_dirty: bool,
    pub buffer: source::Buffer,
    pub editor_view: source::View,
    pub editor_scroll: gtk::ScrolledWindow,
    pub preview: webkit6::WebView,
    pub paned: gtk::Paned,
    pub nav_history: Vec<NavState>,
    pub nav_index: usize,
    pub is_navigating: bool,
    pub tab_page: adw::TabPage,
}

#[allow(dead_code)]
pub struct AppState {
    pub config: Config,
    pub toolbar: Option<gtk::Box>,
    pub cursor_label: Option<gtk::Label>,
    pub preview_toggle: Option<gtk::ToggleButton>,
    pub editor_toggle: Option<gtk::ToggleButton>,
    pub undo_btn: Option<gtk::Button>,
    pub redo_btn: Option<gtk::Button>,
    pub save_btn: Option<gtk::Button>,
    pub local_only_btn: Option<gtk::ToggleButton>,
    pub preview_color_scheme: i32, // 0: System, 1: Light, 2: Dark
    pub css_provider: gtk::CssProvider,
    pub recents_menu: Option<gio::Menu>,
    pub tab_view: adw::TabView,
    pub tab_bar: adw::TabBar,
    pub main_stack: gtk::Stack,
    pub open_tabs: Vec<Rc<RefCell<TabState>>>,
}

impl AppState {
    pub fn get_active_tab(&self) -> Option<Rc<RefCell<TabState>>> {
        let page = self.tab_view.selected_page()?;
        self.open_tabs.iter().find(|t| t.borrow().tab_page == page).cloned()
    }
}

#[allow(dead_code)]
pub struct App {
    pub window: adw::ApplicationWindow,
    pub state: Rc<RefCell<AppState>>,
}
