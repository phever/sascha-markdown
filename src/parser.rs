use crate::config::Config;
use pulldown_cmark::{html, Options, Parser, Event, Tag, TagEnd};

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

fn lookup_emoji(name: &str) -> Option<&'static str> {
    match name {
        "smile" | "slightly_smiling_face" => Some("🙂"),
        "grinning" | "grin" => Some("😀"),
        "joy" | "laughing" => Some("😂"),
        "rofl" => Some("🤣"),
        "smiley" => Some("😃"),
        "blush" => Some("😊"),
        "wink" => Some("😉"),
        "heart_eyes" => Some("😍"),
        "kiss" => Some("😘"),
        "yum" => Some("😋"),
        "sunglasses" | "cool" => Some("😎"),
        "thinking" => Some("🤔"),
        "raised_eyebrow" => Some("🤨"),
        "neutral_face" => Some("😐"),
        "expressionless" => Some("😑"),
        "unamused" => Some("😒"),
        "roll_eyes" => Some("🙄"),
        "grimacing" => Some("😬"),
        "zipper_mouth" => Some("🤐"),
        "hushed" => Some("😯"),
        "flushed" => Some("😳"),
        "confounded" => Some("😖"),
        "disappointed" => Some("😞"),
        "worried" | "concerned" => Some("😟"),
        "cry" | "crying" => Some("😢"),
        "sob" => Some("😭"),
        "scream" => Some("😱"),
        "angry" | "rage" => Some("😡"),
        "triumph" => Some("😤"),
        "skull" | "dead" => Some("💀"),
        "poop" => Some("💩"),
        "clown" => Some("🤡"),
        "ghost" => Some("👻"),
        "alien" => Some("👽"),
        "robot" => Some("🤖"),
        "wave" => Some("👋"),
        "raised_hand" => Some("✋"),
        "ok_hand" => Some("👌"),
        "thumbsup" | "+1" | "thumbup" => Some("👍"),
        "thumbsdown" | "-1" | "thumbdown" => Some("👎"),
        "clap" => Some("👏"),
        "pray" => Some("🙏"),
        "point_right" => Some("👉"),
        "point_left" => Some("👈"),
        "point_up" => Some("☝️"),
        "point_down" => Some("👇"),
        "muscle" | "strong" => Some("💪"),
        "eyes" => Some("👀"),
        "heart" | "love" => Some("❤️"),
        "orange_heart" => Some("🧡"),
        "yellow_heart" => Some("💛"),
        "green_heart" => Some("💚"),
        "blue_heart" => Some("💙"),
        "purple_heart" => Some("💜"),
        "broken_heart" => Some("💔"),
        "star" => Some("⭐"),
        "sparkles" => Some("✨"),
        "fire" | "flame" => Some("🔥"),
        "tada" | "party" => Some("🎉"),
        "trophy" => Some("🏆"),
        "medal" => Some("🥇"),
        "rocket" => Some("🚀"),
        "boom" | "explosion" => Some("💥"),
        "warning" | "warn" => Some("⚠️"),
        "stop" | "prohibited" => Some("🚫"),
        "check" | "white_check_mark" => Some("✅"),
        "x" | "cross" => Some("❌"),
        "question" => Some("❓"),
        "exclamation" | "!" => Some("❗"),
        "info" => Some("ℹ️"),
        "bulb" | "idea" => Some("💡"),
        "gear" | "settings" => Some("⚙️"),
        "lock" => Some("🔒"),
        "unlock" => Some("🔓"),
        "key" => Some("🔑"),
        "link" => Some("🔗"),
        "email" | "mail" => Some("📧"),
        "phone" => Some("📱"),
        "computer" | "laptop" => Some("💻"),
        "keyboard" => Some("⌨️"),
        "mouse" => Some("🖱️"),
        "folder" => Some("📁"),
        "file" | "page" => Some("📄"),
        "pencil" | "edit" => Some("✏️"),
        "clipboard" => Some("📋"),
        "calendar" => Some("📅"),
        "clock" | "time" => Some("🕐"),
        "hourglass" => Some("⏳"),
        "search" | "mag" => Some("🔍"),
        "book" => Some("📖"),
        "books" => Some("📚"),
        "memo" | "note" => Some("📝"),
        "chart" | "graph" => Some("📊"),
        "money" | "cash" => Some("💰"),
        "sun" => Some("☀️"),
        "moon" => Some("🌙"),
        "cloud" => Some("☁️"),
        "rain" => Some("🌧️"),
        "snow" => Some("❄️"),
        "lightning" | "zap" => Some("⚡"),
        "cat" => Some("🐱"),
        "dog" => Some("🐶"),
        "pizza" => Some("🍕"),
        "coffee" => Some("☕"),
        "beer" => Some("🍺"),
        "wine" => Some("🍷"),
        "tada2" => Some("🎊"),
        "music" => Some("🎵"),
        "art" => Some("🎨"),
        "flag" => Some("🏳️"),
        "world" | "earth" => Some("🌍"),
        "house" | "home" => Some("🏠"),
        "car" => Some("🚗"),
        "airplane" => Some("✈️"),
        _ => None,
    }
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
                    output.push_str(emoji_char);
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
            let already_open = stack.iter().any(|(t, _, _, _, _)| t == "NamedQuote");
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
            let already_open = stack.iter().any(|(t, _, _, _, _)| t == "Collapse");
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
/// Block-level tags that get a `data-src-line` attribute injected.
fn is_block_open(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Paragraph
            | Tag::Heading { .. }
            | Tag::BlockQuote(_)
            | Tag::CodeBlock(_)
            | Tag::List(_)
            | Tag::Item
            | Tag::Table(_)
    )
}

fn tag_html_open(tag: &Tag, src_line: u32) -> String {
    match tag {
        Tag::Paragraph => format!(r#"<p data-src-line="{src_line}">"#),
        Tag::Heading { level, .. } => format!(r#"<{level} data-src-line="{src_line}">"#),
        Tag::BlockQuote(_) => format!(r#"<blockquote data-src-line="{src_line}">"#),
        Tag::CodeBlock(_) => format!(r#"<pre data-src-line="{src_line}"><code>"#),
        Tag::List(Some(start)) => format!(r#"<ol start="{start}" data-src-line="{src_line}">"#),
        Tag::List(None) => format!(r#"<ul data-src-line="{src_line}">"#),
        Tag::Item => format!(r#"<li data-src-line="{src_line}">"#),
        Tag::Table(_) => format!(r#"<table data-src-line="{src_line}">"#),
        _ => String::new(),
    }
}

fn tag_html_close(end: &TagEnd) -> &'static str {
    match end {
        TagEnd::Paragraph => "</p>",
        TagEnd::Heading(_) => "",   // handled specially below
        TagEnd::BlockQuote => "</blockquote>",
        TagEnd::CodeBlock => "</code></pre>",
        TagEnd::List(true) => "</ol>",
        TagEnd::List(false) => "</ul>",
        TagEnd::Item => "</li>",
        TagEnd::Table => "</table>",
        _ => "",
    }
}

pub fn render_to_html(text: &str, config: &Config) -> String {
    let preprocessed = preprocess_smd(text, config);
    let options = Options::all();

    // Build a byte→line map so we can tag each block with its source line.
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(preprocessed.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let byte_to_line = |byte: usize| -> u32 {
        line_starts.partition_point(|&s| s <= byte).saturating_sub(1) as u32 + 1
    };

    let parser = Parser::new_ext(&preprocessed, options).into_offset_iter();
    let mut body = String::new();
    let mut heading_level: Option<pulldown_cmark::HeadingLevel> = None;

    for (event, range) in parser {
        match event {
            Event::Start(ref tag) if is_block_open(tag) => {
                let line = byte_to_line(range.start);
                if let Tag::Heading { level, .. } = tag {
                    heading_level = Some(*level);
                    body.push_str(&format!(r#"<{level} data-src-line="{line}">"#));
                } else {
                    body.push_str(&tag_html_open(tag, line));
                }
            }
            Event::End(TagEnd::Heading(level)) => {
                body.push_str(&format!("</{level}>"));
                heading_level = None;
            }
            Event::End(ref end) => {
                let close = tag_html_close(end);
                if !close.is_empty() {
                    body.push_str(close);
                } else {
                    // Fall back to pulldown-cmark for everything else
                    let mut tmp = String::new();
                    html::push_html(&mut tmp, std::iter::once(Event::End(end.clone())));
                    body.push_str(&tmp);
                }
            }
            other => {
                let mut tmp = String::new();
                html::push_html(&mut tmp, std::iter::once(other));
                body.push_str(&tmp);
            }
        }
    }

    let _ = heading_level; // suppress unused warning
    body
}

/// Wraps a body fragment in a complete HTML document, injecting the provided CSS.
pub fn build_html_document(body: &str, css: &str, mode: i32, highlight_color: &str, local_only: bool) -> String {
    let mode_class = match mode {
        1 => "light-mode",
        2 => "dark-mode",
        _ => "",
    };

    let mark_color = if highlight_color.is_empty() {
        String::new()
    } else {
        format!("mark {{ background-color: {}; }}", highlight_color)
    };

    let csp_tag = if local_only {
        r#"<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src file: data: blob:; font-src file: data:;">"#
    } else {
        ""
    };

    format!(
        r#"<!DOCTYPE html>
<html class="{mode_class}">
<head>
<meta charset="utf-8">
<meta name="color-scheme" content="light dark">
{csp_tag}
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

sup.footnote {{
    font-size: 0.75em;
    background: #e0e0e0;
    color: #333;
    border-radius: 3px;
    padding: 0 3px;
    margin: 0 1px;
}}
html.dark-mode sup.footnote {{ background: #444; color: #eee; }}

blockquote.named-quote {{
    border-left: 4px solid #888;
    margin: 0.5em 0;
    padding: 0.5em 1em;
    font-style: italic;
}}
blockquote.named-quote cite.quote-author {{
    display: block;
    font-style: normal;
    font-weight: bold;
    font-size: 0.9em;
    color: #666;
    margin-bottom: 0.25em;
}}
html.dark-mode blockquote.named-quote cite.quote-author {{ color: #aaa; }}

details {{
    border: 1px solid #ccc;
    border-radius: 4px;
    padding: 0.5em 1em;
    margin: 0.5em 0;
}}
details > summary {{
    cursor: pointer;
    font-weight: bold;
    list-style: none;
    padding: 0.25em 0;
}}
details > summary::before {{ content: "▶ "; font-size: 0.8em; }}
details[open] > summary::before {{ content: "▼ "; font-size: 0.8em; }}
html.dark-mode details {{ border-color: #555; }}

{mark_color}
{css}
.error {{ text-decoration: underline wavy red; }}
.warning {{ color: red; font-weight: bold; border: 1px solid red; padding: 4px 8px; border-radius: 4px; margin-bottom: 1em; display: inline-block; }}
[data-src-line].sfmde-cursor-line {{
    outline: 2px solid rgba(100,140,255,0.55);
    outline-offset: 2px;
    border-radius: 3px;
}}
</style>
<script>
(function() {{
    var saved = sessionStorage.getItem('sfmde_scrollY');
    if (saved) {{
        document.addEventListener('DOMContentLoaded', function() {{
            window.scrollTo(0, parseInt(saved, 10));
        }});
    }}
    window.addEventListener('scroll', function() {{
        sessionStorage.setItem('sfmde_scrollY', window.scrollY);
    }}, {{ passive: true }});

    window._sfmde_setCursor = function(line) {{
        var prev = document.querySelector('.sfmde-cursor-line');
        if (prev) prev.classList.remove('sfmde-cursor-line');
        var all = Array.from(document.querySelectorAll('[data-src-line]'));
        if (!all.length) return;
        var best = all[0];
        for (var i = 0; i < all.length; i++) {{
            var l = parseInt(all[i].getAttribute('data-src-line'), 10);
            if (l <= line) best = all[i]; else break;
        }}
        best.classList.add('sfmde-cursor-line');
    }};

    window._sfmde_syncScroll = function(fraction) {{
        var max = document.documentElement.scrollHeight - window.innerHeight;
        if (max > 0) window.scrollTo(0, fraction * max);
    }};
}})();
</script>
</head>
<body>
{body}
</body>
</html>"#,
        mode_class = mode_class,
        mark_color = mark_color,
        css = css,
        body = body
    )
}
