use crate::config::Config;
use crate::parser::emoji::lookup_emoji;
use pulldown_cmark::{Options, Parser, Event, Tag};

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn match_at(chars: &[char], start: usize, pattern: &str) -> bool {
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

pub struct StackEntry {
    pub tag_name: String,
    pub html_open: String,
    pub html_close: String,
    pub output_start: usize,
    pub raw_symbol: String,
    pub original_byte: usize,
}

pub struct PreprocessorOutput {
    pub output: String,
    pub map: Vec<usize>,
}

impl PreprocessorOutput {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            map: Vec::new(),
        }
    }

    pub fn push_char(&mut self, c: char, original_byte: usize) {
        let len = c.len_utf8();
        for _ in 0..len {
            self.map.push(original_byte);
        }
        self.output.push(c);
    }

    pub fn push_str(&mut self, s: &str, original_byte: usize) {
        for c in s.chars() {
            self.push_char(c, original_byte);
        }
    }

    pub fn push_segment(&mut self, s: &str, start_char_idx: usize, char_to_byte: &[usize]) {
        for (idx, c) in s.chars().enumerate() {
            let orig_byte = char_to_byte[start_char_idx + idx];
            self.push_char(c, orig_byte);
        }
    }

    pub fn replace_range_with_mapped(&mut self, range: std::ops::Range<usize>, new_str: &str, original_byte: usize) {
        let new_len = new_str.len();
        self.output.replace_range(range.clone(), new_str);
        self.map.splice(range, std::iter::repeat(original_byte).take(new_len));
    }
}

pub fn preprocess_smd(text: &str, config: &Config) -> (String, Vec<usize>) {
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
    let mut out_writer = PreprocessorOutput::new();
    // Stack: entries tracking open formatting tags
    let mut stack: Vec<StackEntry> = Vec::new();

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
            out_writer.push_segment(&segment, i, &char_to_byte);
            i = end_char;
            continue;
        }

        // Emoji shortcode: :name: — handled before the stack-based system
        if !config.formatters.emoji_prefix.symbol.is_empty()
            && match_at(&chars, i, &config.formatters.emoji_prefix.symbol)
        {
            let sym_len = config.formatters.emoji_prefix.symbol.chars().count();
            let start = i + sym_len;
            let mut end = start;
            while end < chars.len()
                && !match_at(&chars, end, &config.formatters.emoji_prefix.symbol)
                && chars[end] != '\n'
                && chars[end] != ' '
            {
                end += 1;
            }
            if end < chars.len() && match_at(&chars, end, &config.formatters.emoji_prefix.symbol) {
                let name: String = chars[start..end].iter().collect();
                if let Some(emoji_char) = lookup_emoji(&name) {
                    out_writer.push_str(emoji_char, current_byte);
                    i = end + sym_len;
                    continue;
                }
            }
            // No emoji found — fall through
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
        } else if match_at(&chars, i, &config.formatters.footnote.symbol) {
            matched_tag = Some("Footnote".to_string());
            html_open = r#"<sup class="footnote">"#.to_string();
            html_close = "</sup>".to_string();
            skip = config.formatters.footnote.symbol.chars().count();
        } else if !config.formatters.font_color.symbol.is_empty()
            && match_at(&chars, i, &config.formatters.font_color.symbol)
        {
            let sym_len = config.formatters.font_color.symbol.chars().count();
            let start = i + sym_len;
            let mut end = start;
            while end < chars.len() && chars[end] != ' ' && chars[end] != ']' {
                end += 1;
            }
            let color: String = chars[start..end].iter().collect();
            matched_tag = Some("FontColor".to_string());
            html_open = format!("<span style=\"color: {}\">", xml_escape(&color));
            html_close = "</span>".to_string();
            skip = end - i;
        } else if !config.formatters.font_size_change.symbol.is_empty()
            && match_at(&chars, i, &config.formatters.font_size_change.symbol)
        {
            let sym_len = config.formatters.font_size_change.symbol.chars().count();
            let start = i + sym_len;
            let mut end = start;
            while end < chars.len() && chars[end] != ' ' && chars[end] != ']' {
                end += 1;
            }
            let size: String = chars[start..end].iter().collect();
            matched_tag = Some("FontSize".to_string());
            html_open = format!("<span style=\"font-size: {}pt\">", xml_escape(&size));
            html_close = "</span>".to_string();
            skip = end - i;
        } else if !config.formatters.named_quote.symbol.is_empty()
            && match_at(&chars, i, &config.formatters.named_quote.symbol)
        {
            let sym_len = config.formatters.named_quote.symbol.chars().count();
            let already_open = stack.iter().any(|entry| entry.tag_name == "NamedQuote");
            if already_open {
                matched_tag = Some("NamedQuote".to_string());
                skip = sym_len;
            } else {
                let start = i + sym_len;
                let mut end = start;
                while end < chars.len() && chars[end] != ' ' && chars[end] != '\n' {
                    end += 1;
                }
                let author: String = chars[start..end].iter().collect();
                matched_tag = Some("NamedQuote".to_string());
                html_open = format!(
                    r#"<blockquote class="named-quote"><cite class="quote-author">{}</cite> "#,
                    xml_escape(&author)
                );
                html_close = "</blockquote>".to_string();
                skip = if end < chars.len() { end - i + 1 } else { end - i };
            }
        } else if !config.formatters.collapse.symbol.is_empty()
            && match_at(&chars, i, &config.formatters.collapse.symbol)
        {
            let sym_len = config.formatters.collapse.symbol.chars().count();
            let already_open = stack.iter().any(|entry| entry.tag_name == "Collapse");
            if already_open {
                matched_tag = Some("Collapse".to_string());
                skip = sym_len;
            } else {
                let start = i + sym_len;
                let mut end = start;
                while end < chars.len() && chars[end] != ' ' && chars[end] != '\n' {
                    end += 1;
                }
                let title: String = chars[start..end].iter().collect();
                matched_tag = Some("Collapse".to_string());
                html_open = format!(
                    "<details><summary>{}</summary>",
                    xml_escape(&title)
                );
                html_close = "</details>".to_string();
                skip = if end < chars.len() { end - i + 1 } else { end - i };
            }
        } else if !config.formatters.align_left.symbol.is_empty()
            && match_at(&chars, i, &config.formatters.align_left.symbol)
        {
            matched_tag = Some("AlignLeft".to_string());
            html_open = r#"<span style="display:block;text-align:left">"#.to_string();
            html_close = "</span>".to_string();
            skip = config.formatters.align_left.symbol.chars().count();
        } else if !config.formatters.align_right.symbol.is_empty()
            && match_at(&chars, i, &config.formatters.align_right.symbol)
        {
            matched_tag = Some("AlignRight".to_string());
            html_open = r#"<span style="display:block;text-align:right">"#.to_string();
            html_close = "</span>".to_string();
            skip = config.formatters.align_right.symbol.chars().count();
        } else if !config.formatters.align_center.symbol.is_empty()
            && match_at(&chars, i, &config.formatters.align_center.symbol)
        {
            matched_tag = Some("AlignCenter".to_string());
            html_open = r#"<span style="display:block;text-align:center">"#.to_string();
            html_close = "</span>".to_string();
            skip = config.formatters.align_center.symbol.chars().count();
        } else if !config.formatters.align_justify.symbol.is_empty()
            && match_at(&chars, i, &config.formatters.align_justify.symbol)
        {
            matched_tag = Some("AlignJustify".to_string());
            html_open = r#"<span style="display:block;text-align:justify">"#.to_string();
            html_close = "</span>".to_string();
            skip = config.formatters.align_justify.symbol.chars().count();
        }

        if let Some(tag) = matched_tag {
            let raw_symbol: String = chars[i..i + skip].iter().collect();
            let found_index = stack.iter().rposition(|entry| entry.tag_name == tag);

            if let Some(idx) = found_index {
                if idx == stack.len() - 1 {
                    let entry = stack.pop().unwrap();
                    out_writer.push_str(&entry.html_close, current_byte);
                } else {
                    while stack.len() > idx + 1 {
                        let entry = stack.pop().unwrap();
                        let open_len = entry.html_open.len();
                        let error_open = format!(r#"<span class="error">{}"#, entry.raw_symbol);
                        out_writer.replace_range_with_mapped(
                            entry.output_start..entry.output_start + open_len,
                            &error_open,
                            entry.original_byte,
                        );
                        out_writer.push_str("</span>", current_byte);
                    }
                    let entry = stack.pop().unwrap();
                    out_writer.push_str(&entry.html_close, current_byte);
                }
            } else {
                let output_start = out_writer.output.len();
                out_writer.push_str(&html_open, current_byte);
                stack.push(StackEntry {
                    tag_name: tag,
                    html_open,
                    html_close,
                    output_start,
                    raw_symbol,
                    original_byte: current_byte,
                });
            }
            i += skip;
        } else {
            out_writer.push_char(chars[i], current_byte);
            i += 1;
        }
    }

    // Unclosed tags: convert them to error spans
    while let Some(entry) = stack.pop() {
        let open_len = entry.html_open.len();
        let error_open = format!(r#"<span class="error">{}"#, entry.raw_symbol);
        out_writer.replace_range_with_mapped(
            entry.output_start..entry.output_start + open_len,
            &error_open,
            entry.original_byte,
        );
        let end_byte = text.len();
        out_writer.push_str("</span>", end_byte);
    }

    out_writer.map.push(text.len());

    (out_writer.output, out_writer.map)
}
