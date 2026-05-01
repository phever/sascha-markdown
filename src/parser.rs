use crate::config::Config;
use pulldown_cmark::{html, Options, Parser, Event, Tag};

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn match_at(chars: &[char], start: usize, pattern: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    if pattern_chars.is_empty() || start + pattern_chars.len() > chars.len() {
        return false;
    }
    for j in 0..pattern_chars.len() {
        if chars[start + j] != pattern_chars[j] {
            return false;
        }
    }
    true
}

/// Pre-processes the text to replace SMD tags with HTML equivalents,
/// while respecting GFM code blocks and inline code.
fn preprocess_smd(text: &str, config: &Config) -> String {
    let mut excluded_ranges = Vec::new();
    let parser = Parser::new_ext(text, Options::all());
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) | Event::Code(_) | Event::Html(_) | Event::InlineHtml(_) => {
                excluded_ranges.push(range);
            }
            _ => {}
        }
    }

    let chars: Vec<char> = text.chars().collect();
    let mut char_to_byte = Vec::with_capacity(chars.len() + 1);
    let mut byte_offset = 0;
    for c in &chars {
        char_to_byte.push(byte_offset);
        byte_offset += c.len_utf8();
    }
    char_to_byte.push(byte_offset);

    let mut i = 0;
    let mut output = String::new();
    // Stack: (tag_name, html_open, html_close, output_start_index, char_start)
    let mut stack: Vec<(String, String, String, usize, usize)> = Vec::new();

    while i < chars.len() {
        let current_byte = char_to_byte[i];
        
        // Check if we are in an excluded range (code block, inline code, etc.)
        if let Some(range) = excluded_ranges.iter().find(|r| r.start <= current_byte && r.end > current_byte) {
            let end_byte = range.end;
            let mut end_char = i;
            while end_char < chars.len() && char_to_byte[end_char] < end_byte {
                end_char += 1;
            }
            
            let segment: String = chars[i..end_char].iter().collect();
            output.push_str(&segment);
            i = end_char;
            continue;
        }

        let mut matched_tag: Option<String> = None;
        let mut skip = 0;
        let mut html_open = String::new();
        let mut html_close = String::new();

        if match_at(&chars, i, &config.formatters.bold.symbol) {
            matched_tag = Some("Bold".to_string());
            html_open = "<strong>".to_string();
            html_close = "</strong>".to_string();
            skip = config.formatters.bold.symbol.chars().count();
        } else if match_at(&chars, i, &config.formatters.italics.symbol) {
            matched_tag = Some("Italics".to_string());
            html_open = "<em>".to_string();
            html_close = "</em>".to_string();
            skip = config.formatters.italics.symbol.chars().count();
        } else if match_at(&chars, i, &config.formatters.underscore.symbol) {
            matched_tag = Some("Underscore".to_string());
            html_open = "<u>".to_string();
            html_close = "</u>".to_string();
            skip = config.formatters.underscore.symbol.chars().count();
        } else if match_at(&chars, i, &config.formatters.strikethrough.symbol) {
            matched_tag = Some("Strikethrough".to_string());
            html_open = "<s>".to_string();
            html_close = "</s>".to_string();
            skip = config.formatters.strikethrough.symbol.chars().count();
        } else if match_at(&chars, i, &config.formatters.spoiler.symbol) {
            matched_tag = Some("Spoiler".to_string());
            html_open = r#"<span class="spoiler">"#.to_string();
            html_close = "</span>".to_string();
            skip = config.formatters.spoiler.symbol.chars().count();
        } else if match_at(&chars, i, &config.formatters.highlight.symbol) {
            matched_tag = Some("Highlight".to_string());
            html_open = "<mark>".to_string();
            html_close = "</mark>".to_string();
            skip = config.formatters.highlight.symbol.chars().count();
        } else if match_at(&chars, i, &config.formatters.superscript.symbol) {
            matched_tag = Some("Superscript".to_string());
            html_open = "<sup>".to_string();
            html_close = "</sup>".to_string();
            skip = config.formatters.superscript.symbol.chars().count();
        } else if match_at(&chars, i, &config.formatters.subscript.symbol) {
            matched_tag = Some("Subscript".to_string());
            html_open = "<sub>".to_string();
            html_close = "</sub>".to_string();
            skip = config.formatters.subscript.symbol.chars().count();
        } else if match_at(&chars, i, &config.formatters.font_color.symbol) {
            let start = i + config.formatters.font_color.symbol.chars().count();
            let mut end = start;
            while end < chars.len() && chars[end] != ' ' && chars[end] != ']' {
                end += 1;
            }
            let color: String = chars[start..end].iter().collect();
            matched_tag = Some("FontColor".to_string());
            html_open = format!("<span style=\"color: {}\">", xml_escape(&color));
            html_close = "</span>".to_string();
            skip = end - i;
        }

        if let Some(tag) = matched_tag {
            let found_index = stack.iter().rposition(|(t, _, _, _, _)| t == &tag);

            if let Some(idx) = found_index {
                if idx == stack.len() - 1 {
                    let (_, _, html_c, _, _) = stack.pop().unwrap();
                    output.push_str(&html_c);
                } else {
                    while stack.len() > idx {
                        let (_, _, html_c, _, _) = stack.pop().unwrap();
                        output.push_str(&html_c);
                    }
                }
            } else {
                let output_start = output.len();
                output.push_str(&html_open);
                stack.push((tag, html_open, html_close, output_start, i));
            }
            i += skip;
        } else {
            output.push(chars[i]);
            i += 1;
        }
    }

    // Unclosed tags: backtrack and emit raw text as an error span.
    while let Some((_, _, _, output_start, char_start)) = stack.pop() {
        output.truncate(output_start);
        let raw: String = chars[char_start..].iter().collect();
        output.push_str(&format!(r#"<span class="error">{}</span>"#, xml_escape(&raw)));
    }

    output
}

/// Renders SFM-flavoured markdown to an HTML body fragment.
/// Wrap the result with `build_html_document` before loading into a WebView.
pub fn render_to_html(text: &str, config: &Config) -> String {
    let preprocessed = preprocess_smd(text, config);
    let options = Options::all();
    let mut body = String::new();
    html::push_html(&mut body, Parser::new_ext(&preprocessed, options));
    body
}

/// Wraps a body fragment in a complete HTML document, injecting the provided CSS.
pub fn build_html_document(body: &str, css: &str, mode: i32) -> String {
    let mode_class = match mode {
        1 => "light-mode",
        2 => "dark-mode",
        _ => "",
    };

    format!(
        r#"<!DOCTYPE html>
<html class="{mode_class}">
<head>
<meta charset="utf-8">
<meta name="color-scheme" content="light dark">
<style>
:root {{ color-scheme: light dark; }}
html.light-mode {{ color-scheme: light; --bg: white; --fg: black; }}
html.dark-mode {{ color-scheme: dark; --bg: #1e1e1e; --fg: #e0e0e0; }}

body {{
    background-color: var(--bg);
    color: var(--fg);
    margin: 1em auto;
    max-width: 800px;
    padding: 0 1em;
    line-height: 1.6;
}}

html.light-mode body {{ background-color: white; color: black; }}
html.dark-mode body {{ background-color: #1e1e1e; color: #e0e0e0; }}

{css}
.error {{ text-decoration: underline wavy red; }}
.warning {{ color: red; font-weight: bold; border: 1px solid red; padding: 4px 8px; border-radius: 4px; margin-bottom: 1em; display: inline-block; }}
</style>
</head>
<body>
{body}
</body>
</html>"#,
        mode_class = mode_class,
        css = css,
        body = body
    )
}
