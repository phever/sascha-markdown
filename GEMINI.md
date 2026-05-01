# Sascha Flavored Markdown Desktop Editor (SFMDE)

## Project Overview
SFMDE is a high-performance, GTK4 and Libadwaita-based desktop application for authoring "Sascha Flavored Markdown" (SMD). 
SMD is a strict superset of GitHub Flavored Markdown (GFM) that allows users to completely redefine the syntax tokens used for formatting (e.g., changing bold from `**` to `!!`, or highlight from `==` to `^^`) while maintaining 100% standard GFM compliance under the hood.

The project is written in Rust, leveraging `sourceview5` for the text editor and `pulldown-cmark` for the base Markdown parsing.

## Building and Running
The application uses standard Cargo commands:
- **Build**: `cargo build --release`
- **Run**: `cargo run`
- **Test**: `cargo test`

### System Dependencies
To build the project on Linux, you must have the development headers for GTK4, Libadwaita, GtkSourceView5, and WebKitGTK 6.0 installed:
- Arch Linux: `sudo pacman -S --needed gtk4 libadwaita gtksourceview5 webkit2gtk-6.0 pkgconf base-devel`
- Ubuntu/Debian: `sudo apt install libgtk-4-dev libadwaita-1-dev libgtksourceview-5-dev libwebkitgtk-6.0-dev libsoup-3.0-dev pkg-config build-essential`

## Development Architecture
1. **Core Parser (`src/parser.rs`)**: 
   - Utilizes `pulldown-cmark` to parse the document as standard GFM, resulting in a stream of Markdown events.
   - A custom processing pass (`parse_smd_inlines_to_html`) intercepts text nodes and parses SMD-specific inline syntax (e.g., spoiler `||`, highlight `==`, font colors, sizes) based on the user's configuration.
   - The final event stream is translated into a complete HTML document, which is then rendered by a `webkit6::WebView` in the live preview pane.
2. **Configuration Engine (`src/config.rs`)**:
   - Manages formatting tokens, application hotkeys, and appearance settings.
   - Features a cascading configuration hierarchy: falling back from a local `.smdconfig` (per-directory) to the global `~/.config/sascha-flavored-markdown/sfmde.config`.
3. **UI Engine (`src/ui/`)**:
   - `app.rs`: Application initialization, window layout, GTK actions, and CSS loading.
   - `markup.rs`: Intelligent text manipulation logic for wrapping, unwrapping, and applying multi-line list markers in the editor buffer.
   - `toolbar.rs`: Adaptive formatting toolbar that dynamically wraps into an overflow menu based on window width.
   - `settings.rs`: Comprehensive `adw::PreferencesWindow` GUI to edit configurations.

## Development Conventions
- **UI Toolkit**: Strict adherence to `gtk4-rs` and `libadwaita` patterns (using `adw::ApplicationWindow`, `adw::HeaderBar`, `adw::PreferencesWindow`).
- **State Management**: Application state is managed via `Rc<RefCell<AppState>>` passed into closure callbacks. 
- **Thread Safety**: Callbacks utilize `glib::clone!` or manual clones with `move` to capture state safely. Polling is done safely on the main thread via `glib::timeout_add_local`.
