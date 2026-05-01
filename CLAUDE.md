# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                # debug build
cargo build --release      # release build (binary: target/release/sfmde)
cargo run                  # run the app
cargo check                # type-check without building
cargo clippy               # lint
```

There are no automated tests. The app requires a display to run (GTK4).

## Architecture

SFMDE is a GTK4/libadwaita desktop Markdown editor written in Rust. It edits `.smd` (Sascha Flavored Markdown) files with a dual-pane editor + live preview.

### Module layout

- **`src/main.rs`** — entry point; calls `config::ensure_config_exists()` on first run, constructs the `adw::Application`, shows a welcome dialog if config was just created.
- **`src/config.rs`** — all config structs (`FormatterConfig`, `HotkeysConfig`, `AppearanceConfig`, `Config`) serialized as TOML. Two config tiers: global at `~/.config/sascha-flavored-markdown/sfmde.config` and per-directory `.smdconfig`. Local takes precedence over global. Also manages `style.css` (preview CSS).
- **`src/parser.rs`** — custom line-oriented `BetterParser` that produces `Event` items and `render_to_html`. Uses config symbols at parse time so all formatting delimiters are user-configurable. Note: `pulldown-cmark` is listed as a dependency but is not used; the parser is fully custom.
- **`src/ui/mod.rs`** — shared state types: `AppState` (current file, config, nav history, toolbar/buffer refs) and `App` (window, editor, preview). `AppState` is always accessed via `Rc<RefCell<AppState>>`.
- **`src/ui/app.rs`** — entire window construction and all GTK signal wiring. Also `apply_appearance` (dynamic CSS injection for font/color) and `setup_accels` (keyboard shortcut registration).
- **`src/ui/markup.rs`** — `apply_markup`: wraps/unwraps a selection with a symbol, or toggles list prefixes line-by-line.
- **`src/ui/toolbar.rs`** — `refresh_toolbar`: rebuilds the adaptive formatter toolbar. A `glib::timeout_add_local` at 200ms polls window width and calls this when the available button count changes.
- **`src/ui/settings.rs`** — `show_settings_dialog` and `populate_*` helpers for the Adwaita `PreferencesWindow` (formatters, hotkeys, appearance). Each field saves to disk immediately on change.

### Key design points

- **Preview rendering**: The preview pane is a `webkit6::WebView`, not a `gtk::Label`. `render_to_html` produces an HTML body fragment, which is then wrapped into a full HTML document (with injected CSS) and loaded into the WebView. Only `.smd` files get a clean preview; other file types show a warning banner.
- **Formatter symbols are runtime-configurable**: The parser reads delimiter symbols from `Config` at construction time. Changing a symbol in settings rebuilds the toolbar and affects all subsequent parses.
- **Navigation history**: Cursor position changes >100 chars apart (or file changes) are recorded in `nav_history`. Back/Forward buttons walk this history. The `is_navigating` flag suppresses history recording during programmatic cursor moves.
- **`chop.py`**: A one-off script used to split the original monolithic `src/ui.rs` into the current module structure. It is no longer needed.
