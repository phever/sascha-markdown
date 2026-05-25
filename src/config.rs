use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormatterEntry {
    pub symbol: String,
    pub visible: bool,
    pub icon_name: String,
}

impl FormatterEntry {
    pub fn new(symbol: &str, visible: bool, icon_name: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            visible,
            icon_name: icon_name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct FormatterConfig {
    pub italics: FormatterEntry,
    pub bold: FormatterEntry,
    pub underscore: FormatterEntry,
    pub strikethrough: FormatterEntry,
    pub paragraph: FormatterEntry,
    pub preformatted: FormatterEntry,
    pub font_size_change: FormatterEntry,
    pub align_left: FormatterEntry,
    pub align_right: FormatterEntry,
    pub align_center: FormatterEntry,
    pub align_justify: FormatterEntry,
    pub superscript: FormatterEntry,
    pub subscript: FormatterEntry,
    pub code_block: FormatterEntry,
    pub blockquote: FormatterEntry,
    pub nested_blockquote: FormatterEntry,
    pub escape_char: FormatterEntry,
    pub font_color: FormatterEntry,
    pub highlight: FormatterEntry,
    pub table_start: FormatterEntry,
    pub table_row: FormatterEntry,
    pub table_cell: FormatterEntry,
    pub url_link: FormatterEntry,
    pub image_insert: FormatterEntry,
    pub unordered_list: FormatterEntry,
    pub ordered_list: FormatterEntry,
    pub task_list: FormatterEntry,
    pub quote: FormatterEntry,
    pub named_quote: FormatterEntry,
    pub spoiler: FormatterEntry,
    pub horizontal_rule: FormatterEntry,
    pub heading_prefix: FormatterEntry,
    pub heading_id: FormatterEntry,
    pub footnote: FormatterEntry,
    pub emoji_prefix: FormatterEntry,
    pub collapse: FormatterEntry,
}

impl FormatterConfig {
    pub fn all_formatters(&self) -> Vec<(String, String, bool, String)> {
        vec![
            ("Italics".to_string(), self.italics.symbol.clone(), self.italics.visible, self.italics.icon_name.clone()),
            ("Bold".to_string(), self.bold.symbol.clone(), self.bold.visible, self.bold.icon_name.clone()),
            ("Underscore".to_string(), self.underscore.symbol.clone(), self.underscore.visible, self.underscore.icon_name.clone()),
            ("Strikethrough".to_string(), self.strikethrough.symbol.clone(), self.strikethrough.visible, self.strikethrough.icon_name.clone()),
            ("Paragraph".to_string(), self.paragraph.symbol.clone(), self.paragraph.visible, self.paragraph.icon_name.clone()),
            ("Preformatted".to_string(), self.preformatted.symbol.clone(), self.preformatted.visible, self.preformatted.icon_name.clone()),
            ("Font Size Change".to_string(), self.font_size_change.symbol.clone(), self.font_size_change.visible, self.font_size_change.icon_name.clone()),
            ("Align Left".to_string(), self.align_left.symbol.clone(), self.align_left.visible, self.align_left.icon_name.clone()),
            ("Align Right".to_string(), self.align_right.symbol.clone(), self.align_right.visible, self.align_right.icon_name.clone()),
            ("Align Center".to_string(), self.align_center.symbol.clone(), self.align_center.visible, self.align_center.icon_name.clone()),
            ("Align Justify".to_string(), self.align_justify.symbol.clone(), self.align_justify.visible, self.align_justify.icon_name.clone()),
            ("Superscript".to_string(), self.superscript.symbol.clone(), self.superscript.visible, self.superscript.icon_name.clone()),
            ("Subscript".to_string(), self.subscript.symbol.clone(), self.subscript.visible, self.subscript.icon_name.clone()),
            ("Code Block".to_string(), self.code_block.symbol.clone(), self.code_block.visible, self.code_block.icon_name.clone()),
            ("Blockquote".to_string(), self.blockquote.symbol.clone(), self.blockquote.visible, self.blockquote.icon_name.clone()),
            ("Nested Blockquote".to_string(), self.nested_blockquote.symbol.clone(), self.nested_blockquote.visible, self.nested_blockquote.icon_name.clone()),
            ("Escape Char".to_string(), self.escape_char.symbol.clone(), self.escape_char.visible, self.escape_char.icon_name.clone()),
            ("Font Color".to_string(), self.font_color.symbol.clone(), self.font_color.visible, self.font_color.icon_name.clone()),
            ("Highlight".to_string(), self.highlight.symbol.clone(), self.highlight.visible, self.highlight.icon_name.clone()),
            ("Table Start".to_string(), self.table_start.symbol.clone(), self.table_start.visible, self.table_start.icon_name.clone()),
            ("Table Row".to_string(), self.table_row.symbol.clone(), self.table_row.visible, self.table_row.icon_name.clone()),
            ("Table Cell".to_string(), self.table_cell.symbol.clone(), self.table_cell.visible, self.table_cell.icon_name.clone()),
            ("URL Link".to_string(), self.url_link.symbol.clone(), self.url_link.visible, self.url_link.icon_name.clone()),
            ("Image Insert".to_string(), self.image_insert.symbol.clone(), self.image_insert.visible, self.image_insert.icon_name.clone()),
            ("Unordered List".to_string(), self.unordered_list.symbol.clone(), self.unordered_list.visible, self.unordered_list.icon_name.clone()),
            ("Ordered List".to_string(), self.ordered_list.symbol.clone(), self.ordered_list.visible, self.ordered_list.icon_name.clone()),
            ("Task List".to_string(), self.task_list.symbol.clone(), self.task_list.visible, self.task_list.icon_name.clone()),
            ("Quote".to_string(), self.quote.symbol.clone(), self.quote.visible, self.quote.icon_name.clone()),
            ("Named Quote".to_string(), self.named_quote.symbol.clone(), self.named_quote.visible, self.named_quote.icon_name.clone()),
            ("Spoiler".to_string(), self.spoiler.symbol.clone(), self.spoiler.visible, self.spoiler.icon_name.clone()),
            ("Horizontal Rule".to_string(), self.horizontal_rule.symbol.clone(), self.horizontal_rule.visible, self.horizontal_rule.icon_name.clone()),
            ("Heading Prefix".to_string(), self.heading_prefix.symbol.clone(), self.heading_prefix.visible, self.heading_prefix.icon_name.clone()),
            ("Heading ID".to_string(), self.heading_id.symbol.clone(), self.heading_id.visible, self.heading_id.icon_name.clone()),
            ("Footnote".to_string(), self.footnote.symbol.clone(), self.footnote.visible, self.footnote.icon_name.clone()),
            ("Emoji Prefix".to_string(), self.emoji_prefix.symbol.clone(), self.emoji_prefix.visible, self.emoji_prefix.icon_name.clone()),
            ("Collapse".to_string(), self.collapse.symbol.clone(), self.collapse.visible, self.collapse.icon_name.clone()),
        ]
    }

    pub fn update_from_vec(&mut self, formatters: Vec<(String, String, bool, String)>) {
        for (name, symbol, visible, icon_name) in formatters {
            match name.as_str() {
                "Italics" => self.italics = FormatterEntry::new(&symbol, visible, &icon_name),
                "Bold" => self.bold = FormatterEntry::new(&symbol, visible, &icon_name),
                "Underscore" => self.underscore = FormatterEntry::new(&symbol, visible, &icon_name),
                "Strikethrough" => self.strikethrough = FormatterEntry::new(&symbol, visible, &icon_name),
                "Paragraph" => self.paragraph = FormatterEntry::new(&symbol, visible, &icon_name),
                "Preformatted" => self.preformatted = FormatterEntry::new(&symbol, visible, &icon_name),
                "Font Size Change" => self.font_size_change = FormatterEntry::new(&symbol, visible, &icon_name),
                "Align Left" => self.align_left = FormatterEntry::new(&symbol, visible, &icon_name),
                "Align Right" => self.align_right = FormatterEntry::new(&symbol, visible, &icon_name),
                "Align Center" => self.align_center = FormatterEntry::new(&symbol, visible, &icon_name),
                "Align Justify" => self.align_justify = FormatterEntry::new(&symbol, visible, &icon_name),
                "Superscript" => self.superscript = FormatterEntry::new(&symbol, visible, &icon_name),
                "Subscript" => self.subscript = FormatterEntry::new(&symbol, visible, &icon_name),
                "Code Block" => self.code_block = FormatterEntry::new(&symbol, visible, &icon_name),
                "Blockquote" => self.blockquote = FormatterEntry::new(&symbol, visible, &icon_name),
                "Nested Blockquote" => self.nested_blockquote = FormatterEntry::new(&symbol, visible, &icon_name),
                "Escape Char" => self.escape_char = FormatterEntry::new(&symbol, visible, &icon_name),
                "Font Color" => self.font_color = FormatterEntry::new(&symbol, visible, &icon_name),
                "Highlight" => self.highlight = FormatterEntry::new(&symbol, visible, &icon_name),
                "Table Start" => self.table_start = FormatterEntry::new(&symbol, visible, &icon_name),
                "Table Row" => self.table_row = FormatterEntry::new(&symbol, visible, &icon_name),
                "Table Cell" => self.table_cell = FormatterEntry::new(&symbol, visible, &icon_name),
                "URL Link" => self.url_link = FormatterEntry::new(&symbol, visible, &icon_name),
                "Image Insert" => self.image_insert = FormatterEntry::new(&symbol, visible, &icon_name),
                "Unordered List" => self.unordered_list = FormatterEntry::new(&symbol, visible, &icon_name),
                "Ordered List" => self.ordered_list = FormatterEntry::new(&symbol, visible, &icon_name),
                "Task List" => self.task_list = FormatterEntry::new(&symbol, visible, &icon_name),
                "Quote" => self.quote = FormatterEntry::new(&symbol, visible, &icon_name),
                "Named Quote" => self.named_quote = FormatterEntry::new(&symbol, visible, &icon_name),
                "Spoiler" => self.spoiler = FormatterEntry::new(&symbol, visible, &icon_name),
                "Horizontal Rule" => self.horizontal_rule = FormatterEntry::new(&symbol, visible, &icon_name),
                "Heading Prefix" => self.heading_prefix = FormatterEntry::new(&symbol, visible, &icon_name),
                "Heading ID" => self.heading_id = FormatterEntry::new(&symbol, visible, &icon_name),
                "Footnote" => self.footnote = FormatterEntry::new(&symbol, visible, &icon_name),
                "Emoji Prefix" => self.emoji_prefix = FormatterEntry::new(&symbol, visible, &icon_name),
                "Collapse" => self.collapse = FormatterEntry::new(&symbol, visible, &icon_name),
                _ => {}
            }
        }
    }
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            italics: FormatterEntry::new("*", true, "format-text-italic-symbolic"),
            bold: FormatterEntry::new("**", true, "format-text-bold-symbolic"),
            underscore: FormatterEntry::new("_", true, "format-text-underline-symbolic"),
            strikethrough: FormatterEntry::new("~~", true, "format-text-strikethrough-symbolic"),
            paragraph: FormatterEntry::new("\n\n", false, "insert-text-symbolic"),
            preformatted: FormatterEntry::new("```", true, "text-x-generic-symbolic"),
            font_size_change: FormatterEntry::new("size:", false, "format-text-larger-symbolic"),
            align_left: FormatterEntry::new("[:left]", false, "format-justify-left-symbolic"),
            align_right: FormatterEntry::new("[:right]", false, "format-justify-right-symbolic"),
            align_center: FormatterEntry::new("[:center]", false, "format-justify-center-symbolic"),
            align_justify: FormatterEntry::new("[:justify]", false, "format-justify-fill-symbolic"),
            superscript: FormatterEntry::new("^", true, "format-text-superscript-symbolic"),
            subscript: FormatterEntry::new("~", true, "format-text-subscript-symbolic"),
            code_block: FormatterEntry::new("```", true, "applications-engineering-symbolic"),
            blockquote: FormatterEntry::new("> ", true, "format-indent-more-symbolic"),
            nested_blockquote: FormatterEntry::new(">> ", true, "format-indent-more-symbolic"),
            escape_char: FormatterEntry::new("\\", false, "media-playlist-consecutive-symbolic"),
            font_color: FormatterEntry::new("color:", false, "format-fill-color-symbolic"),
            highlight: FormatterEntry::new("==", true, "emblem-favorite-symbolic"),
            table_start: FormatterEntry::new("|", true, "view-grid-symbolic"),
            table_row: FormatterEntry::new("|", true, "view-grid-symbolic"),
            table_cell: FormatterEntry::new("|", true, "view-grid-symbolic"),
            url_link: FormatterEntry::new("[]()", true, "insert-link-symbolic"),
            image_insert: FormatterEntry::new("![]()", true, "insert-image-symbolic"),
            unordered_list: FormatterEntry::new("- ", true, "view-list-symbolic"),
            ordered_list: FormatterEntry::new("1. ", true, "view-sort-ascending-symbolic"),
            task_list: FormatterEntry::new("- [ ] ", true, "emblem-ok-symbolic"),
            quote: FormatterEntry::new("\"", true, "format-indent-more-symbolic"),
            named_quote: FormatterEntry::new("quote:", true, "format-indent-more-symbolic"),
            spoiler: FormatterEntry::new("||", true, "view-conceal-symbolic"),
            horizontal_rule: FormatterEntry::new("---", true, "window-minimize-symbolic"),
            heading_prefix: FormatterEntry::new("#", true, "format-text-larger-symbolic"),
            heading_id: FormatterEntry::new("{#}", false, "tag-symbolic"),
            footnote: FormatterEntry::new("[^]", false, "index-symbolic"),
            emoji_prefix: FormatterEntry::new(":", true, "face-smile-symbolic"),
            collapse: FormatterEntry::new("+++", false, "go-down-symbolic"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HotkeysConfig {
    pub mappings: std::collections::HashMap<String, String>,
}

impl Default for HotkeysConfig {
    fn default() -> Self {
        let mut mappings = std::collections::HashMap::new();
        // Common defaults
        mappings.insert("Bold".to_string(), "<Control>b".to_string());
        mappings.insert("Italics".to_string(), "<Control>i".to_string());
        mappings.insert("Underscore".to_string(), "<Control>u".to_string());
        mappings.insert("Strikethrough".to_string(), "<Control>d".to_string());
        mappings.insert("Save".to_string(), "<Control>s".to_string());
        mappings.insert("Open".to_string(), "<Control>o".to_string());
        mappings.insert("New File".to_string(), "<Control>n".to_string());
        mappings.insert("Undo".to_string(), "<Control>z".to_string());
        mappings.insert("Redo".to_string(), "<Control><Shift>z".to_string());
        
        Self { mappings }
    }
}

impl HotkeysConfig {
    pub fn get(&self, name: &str) -> String {
        self.mappings.get(name).cloned().unwrap_or_default()
    }

    pub fn all_hotkeys(&self, all_formatter_names: &[String]) -> Vec<(String, String)> {
        let mut result = Vec::new();
        // Add hardcoded actions first
        let actions = ["Save", "Open", "New File", "Undo", "Redo"];
        for action in actions {
            result.push((action.to_string(), self.get(action)));
        }
        // Add all formatters
        for name in all_formatter_names {
            result.push((name.clone(), self.get(name)));
        }
        result
    }

    pub fn update(&mut self, name: &str, shortcut: String) {
        self.mappings.insert(name.to_string(), shortcut);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppearanceConfig {
    pub editor_font_family: String,
    pub editor_font_size: u32,
    pub editor_bg_color: String,
    pub editor_fg_color: String,
    pub menu_icon_size: u32,
    pub splitbar_color: String,
    pub splitbar_width: u32,
    pub word_wrap: bool,
    pub show_line_col: bool,
    pub show_line_numbers: bool,
    pub highlight_color: String,
    pub last_open_dir: String,
    pub local_only: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            editor_font_family: "Monospace".to_string(),
            editor_font_size: 14,
            editor_bg_color: "".to_string(),
            editor_fg_color: "".to_string(),
            menu_icon_size: 16,
            splitbar_color: "#ccc".to_string(),
            splitbar_width: 2,
            word_wrap: false,
            show_line_col: true,
            show_line_numbers: true,
            highlight_color: "".to_string(),
            last_open_dir: "".to_string(),
            local_only: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: String,
    pub history_length: usize,
    pub formatters: FormatterConfig,
    pub hotkeys: HotkeysConfig,
    pub appearance: AppearanceConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            history_length: 100,
            formatters: FormatterConfig::default(),
            hotkeys: HotkeysConfig::default(),
            appearance: AppearanceConfig::default(),
        }
    }
}

pub fn get_global_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "sascha", "sascha-flavored-markdown")
        .map(|dirs| dirs.config_dir().join("sfmde.config"))
}

pub fn load_config(current_dir: &Path) -> Result<Config> {
    let global_config = get_global_config_path()
        .and_then(|path| if path.exists() { Some(path) } else { None })
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|content| toml::from_str::<Config>(&content).ok());

    let local_config_path = current_dir.join(".smdconfig");
    if local_config_path.exists() {
        let content = fs::read_to_string(&local_config_path)?;
        let config: Config = toml::from_str(&content).context("Failed to parse local .smdconfig")?;
        
        // Migration check
        if let Some(global) = global_config {
            if config.version != global.version {
                println!("Warning: Local .smdconfig version ({}) differs from global version ({})", config.version, global.version);
                // In a real app, this might trigger a dialog or automatic update
            }
        }
        return Ok(config);
    }

    // 2. Return global config or default
    Ok(global_config.unwrap_or_default())
}

pub fn get_style_css_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "sascha", "sascha-flavored-markdown")
        .map(|dirs| dirs.config_dir().join("style.css"))
}

pub fn ensure_config_exists() -> Result<bool> {
    let mut created = false;
    
    // Ensure dir exists
    if let Some(global_path) = get_global_config_path() {
        if let Some(parent) = global_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).context("Failed to create global config directory")?;
            }
        }
    }

    // Config
    if let Some(global_path) = get_global_config_path() {
        if !global_path.exists() {
            if let Ok(default_content) = fs::read_to_string("res/default.toml") {
                fs::write(global_path, default_content).context("Failed to write default global config")?;
            } else {
                let default_config = Config::default();
                let content = toml::to_string_pretty(&default_config).context("Failed to serialize default config")?;
                fs::write(global_path, content).context("Failed to write default global config")?;
            }
            created = true;
        }
    }
    
    // CSS
    if let Some(css_path) = get_style_css_path() {
        if !css_path.exists() {
            if let Ok(default_css) = fs::read_to_string("res/default.css") {
                fs::write(css_path, default_css).context("Failed to write default style.css")?;
            }
        }
    }
    
    Ok(created)
}

pub fn save_global_config(config: &Config) -> Result<()> {
    if let Some(path) = get_global_config_path() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(config)?;
        fs::write(path, content)?;
    }
    Ok(())
}

pub fn save_local_config(config: &Config, current_dir: &Path) -> Result<()> {
    let path = current_dir.join(".smdconfig");
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}
