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

#[derive(Clone)]
pub struct NavState {
    pub file: Option<PathBuf>,
    pub cursor_offset: i32,
}

pub struct AppState {
    pub current_file: Option<PathBuf>,
    pub config: Config,
    pub nav_history: Vec<NavState>,
    pub nav_index: usize,
    pub is_navigating: bool,
    pub is_dirty: bool,
    pub toolbar: Option<gtk::Box>,
    pub buffer: Option<source::Buffer>,
    pub editor_view: Option<source::View>,
    pub cursor_label: Option<gtk::Label>,
    pub preview_toggle: Option<gtk::ToggleButton>,
    pub editor_visible: bool,
    pub preview_visible: bool,
    pub preview_color_scheme: i32, // 0: System, 1: Light, 2: Dark
    pub css_provider: gtk::CssProvider,
    pub recents_menu: Option<gio::Menu>,
}

use gtk4::gio;

#[allow(dead_code)]
pub struct App {
    #[allow(dead_code)]
    pub window: adw::ApplicationWindow,
    #[allow(dead_code)]
    pub editor: source::View,
    #[allow(dead_code)]
    pub preview: webkit6::WebView,
    #[allow(dead_code)]
    pub state: Rc<RefCell<AppState>>,
}
