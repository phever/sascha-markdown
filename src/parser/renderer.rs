use crate::config::Config;
use crate::parser::preprocessor::{preprocess_smd, xml_escape};
use pulldown_cmark::{html, Options, Parser, Event, Tag, TagEnd};

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
    let options = Options::all();
    let (preprocessed, map) = preprocess_smd(text, config);

    // Map byte offsets in the original text to 1-based line numbers.
    let original_line_starts: Vec<usize> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let byte_to_line = |byte: usize| -> u32 {
        let orig_byte = if byte < map.len() { map[byte] } else { text.len() };
        original_line_starts.partition_point(|&s| s <= orig_byte).saturating_sub(1) as u32 + 1
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
                    let mut tmp = String::new();
                    html::push_html(&mut tmp, std::iter::once(Event::End(end.clone())));
                    body.push_str(&tmp);
                }
            }
            Event::Rule => {
                let line = byte_to_line(range.start);
                body.push_str(&format!(r#"<hr data-src-line="{line}" />"#));
            }
            other => {
                let mut tmp = String::new();
                html::push_html(&mut tmp, std::iter::once(other));
                body.push_str(&tmp);
            }
        }
    }

    let _ = heading_level;
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
span.spoiler {{
    background-color: #b5bac1;
    color: #b5bac1;
    border-radius: 3px;
    padding: 0 3px;
    cursor: pointer;
    user-select: none;
    transition: background-color 0.2s, color 0.2s;
}}
span.spoiler.revealed {{
    background-color: rgba(79,84,92,0.3);
    color: inherit;
    user-select: text;
}}
[data-src-line].sfmde-cursor-line {{
    outline: 2px solid rgba(100,140,255,0.55);
    outline-offset: 2px;
    border-radius: 3px;
}}
</style>
<script>
(function() {{
    try {{
        var saved = sessionStorage.getItem('sfmde_scrollY');
        if (saved) {{
            document.addEventListener('DOMContentLoaded', function() {{
                window.scrollTo(0, parseInt(saved, 10));
            }});
        }}
        window.addEventListener('scroll', function() {{
            try {{ sessionStorage.setItem('sfmde_scrollY', window.scrollY); }} catch(e) {{}}
        }}, {{ passive: true }});
    }} catch(e) {{}}

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

    document.addEventListener('click', function(e) {{
        var s = e.target.closest('.spoiler');
        if (s) s.classList.toggle('revealed');
    }});

    window._sfmde_syncScroll = function(fraction) {{
        var max = document.documentElement.scrollHeight - window.innerHeight;
        if (max > 0) window.scrollTo(0, fraction * max);
    }};
}})(),
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

pub fn render_to_html_inline(text: &str, config: &Config) -> String {
    let options = Options::all();
    let (preprocessed, _) = preprocess_smd(text, config);
    let parser = Parser::new_ext(&preprocessed, options);

    let mut body = String::new();
    let mut table_row_index = 0;
    let mut in_table_head = false;

    let mut image_dest: Option<String> = None;
    let mut image_title: Option<String> = None;
    let mut image_alt: Option<String> = None;

    for event in parser {
        match event {
            Event::Start(tag) => {
                if image_alt.is_some() {
                    continue;
                }
                match tag {
                    Tag::Paragraph => {
                        body.push_str(r#"<p style="margin-top: 0; margin-bottom: 16px; line-height: 1.6;">"#);
                    }
                    Tag::Heading { level, .. } => {
                        let tag_name = match level {
                            pulldown_cmark::HeadingLevel::H1 => "h1",
                            pulldown_cmark::HeadingLevel::H2 => "h2",
                            pulldown_cmark::HeadingLevel::H3 => "h3",
                            pulldown_cmark::HeadingLevel::H4 => "h4",
                            pulldown_cmark::HeadingLevel::H5 => "h5",
                            pulldown_cmark::HeadingLevel::H6 => "h6",
                        };
                        let style = match level {
                            pulldown_cmark::HeadingLevel::H1 => "font-size: 2em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; margin-top: 24px; margin-bottom: 16px; font-weight: 600; color: #24292f;",
                            pulldown_cmark::HeadingLevel::H2 => "font-size: 1.5em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; margin-top: 24px; margin-bottom: 16px; font-weight: 600; color: #24292f;",
                            pulldown_cmark::HeadingLevel::H3 => "font-size: 1.25em; margin-top: 24px; margin-bottom: 16px; font-weight: 600; color: #24292f;",
                            pulldown_cmark::HeadingLevel::H4 => "font-size: 1em; margin-top: 24px; margin-bottom: 16px; font-weight: 600; color: #24292f;",
                            pulldown_cmark::HeadingLevel::H5 => "font-size: 0.875em; margin-top: 24px; margin-bottom: 16px; font-weight: 600; color: #24292f;",
                            pulldown_cmark::HeadingLevel::H6 => "font-size: 0.85em; margin-top: 24px; margin-bottom: 16px; font-weight: 600; color: #6a737d;",
                        };
                        body.push_str(&format!(r#"<{} style="{}">"#, tag_name, style));
                    }
                    Tag::BlockQuote(_) => {
                        body.push_str(r#"<blockquote style="padding: 0 1em; color: #57606a; border-left: .25em solid #d0d7de; margin: 0 0 16px 0;">"#);
                    }
                    Tag::CodeBlock(_) => {
                        body.push_str(r#"<pre style="padding: 16px; overflow: auto; font-size: 85%; line-height: 1.45; background-color: #f6f8fa; border-radius: 6px; margin-top: 0; margin-bottom: 16px;"><code style="background: none; padding: 0; margin: 0; font-size: 100%; word-break: normal; border: 0; display: block; overflow-x: auto; font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;">"#);
                    }
                    Tag::List(Some(start)) => {
                        body.push_str(&format!(r#"<ol start="{}" style="padding-left: 2em; margin-top: 0; margin-bottom: 16px;">"#, start));
                    }
                    Tag::List(None) => {
                        body.push_str(r#"<ul style="padding-left: 2em; margin-top: 0; margin-bottom: 16px;">"#);
                    }
                    Tag::Item => {
                        body.push_str(r#"<li style="margin-top: 0.25em;">"#);
                    }
                    Tag::Emphasis => {
                        body.push_str(r#"<em style="font-style: italic;">"#);
                    }
                    Tag::Strong => {
                        body.push_str(r#"<strong style="font-weight: bold;">"#);
                    }
                    Tag::Strikethrough => {
                        body.push_str(r#"<s style="text-decoration: line-through;">"#);
                    }
                    Tag::Link { dest_url, title, .. } => {
                        let title_attr = if title.is_empty() { String::new() } else { format!(r#" title="{}""#, xml_escape(&title)) };
                        body.push_str(&format!(r#"<a href="{}"{} style="color: #0969da; text-decoration: none;">"#, xml_escape(&dest_url), title_attr));
                    }
                    Tag::Image { dest_url, title, .. } => {
                        image_dest = Some(dest_url.to_string());
                        image_title = Some(title.to_string());
                        image_alt = Some(String::new());
                    }
                    Tag::Table(_) => {
                        body.push_str(r#"<table style="border-spacing: 0; border-collapse: collapse; margin-top: 0; margin-bottom: 16px; width: 100%; display: block; overflow: auto;">"#);
                        table_row_index = 0;
                    }
                    Tag::TableHead => {
                        body.push_str(r#"<thead style="font-weight: 600;">"#);
                        in_table_head = true;
                    }
                    Tag::TableRow => {
                        if in_table_head {
                            body.push_str(r#"<tr>"#);
                        } else {
                            let bg = if table_row_index % 2 == 0 { "#ffffff" } else { "#f6f8fa" };
                            body.push_str(&format!(r#"<tr style="background-color: {};">"#, bg));
                            table_row_index += 1;
                        }
                    }
                    Tag::TableCell => {
                        if in_table_head {
                            body.push_str(r#"<th style="padding: 6px 13px; border: 1px solid #d0d7de; background-color: #f6f8fa; font-weight: 600; text-align: left;">"#);
                        } else {
                            body.push_str(r#"<td style="padding: 6px 13px; border: 1px solid #d0d7de; text-align: left;">"#);
                        }
                    }
                    _ => {
                        let mut tmp = String::new();
                        html::push_html(&mut tmp, std::iter::once(Event::Start(tag)));
                        body.push_str(&tmp);
                    }
                }
            }
            Event::End(tag_end) => {
                if image_alt.is_some() && tag_end != TagEnd::Image {
                    continue;
                }
                match tag_end {
                    TagEnd::Paragraph => body.push_str("</p>"),
                    TagEnd::Heading(level) => {
                        let tag_name = match level {
                            pulldown_cmark::HeadingLevel::H1 => "h1",
                            pulldown_cmark::HeadingLevel::H2 => "h2",
                            pulldown_cmark::HeadingLevel::H3 => "h3",
                            pulldown_cmark::HeadingLevel::H4 => "h4",
                            pulldown_cmark::HeadingLevel::H5 => "h5",
                            pulldown_cmark::HeadingLevel::H6 => "h6",
                        };
                        body.push_str(&format!("</{}>", tag_name));
                    }
                    TagEnd::BlockQuote => body.push_str("</blockquote>"),
                    TagEnd::CodeBlock => body.push_str("</code></pre>"),
                    TagEnd::List(true) => body.push_str("</ol>"),
                    TagEnd::List(false) => body.push_str("</ul>"),
                    TagEnd::Item => body.push_str("</li>"),
                    TagEnd::Emphasis => body.push_str("</em>"),
                    TagEnd::Strong => body.push_str("</strong>"),
                    TagEnd::Strikethrough => body.push_str("</s>"),
                    TagEnd::Link => body.push_str("</a>"),
                    TagEnd::Image => {
                        let dest = image_dest.take().unwrap_or_default();
                        let title = image_title.take().unwrap_or_default();
                        let alt = image_alt.take().unwrap_or_default();
                        let title_attr = if title.is_empty() { String::new() } else { format!(r#" title="{}""#, xml_escape(&title)) };
                        body.push_str(&format!(r#"<img src="{}" alt="{}"{} style="max-width: 100%; box-sizing: border-box; display: block; margin: 16px 0;" />"#, xml_escape(&dest), xml_escape(&alt), title_attr));
                    }
                    TagEnd::Table => body.push_str("</table>"),
                    TagEnd::TableHead => {
                        body.push_str("</thead>");
                        in_table_head = false;
                    }
                    TagEnd::TableRow => body.push_str("</tr>"),
                    TagEnd::TableCell => {
                        if in_table_head {
                            body.push_str("</th>");
                        } else {
                            body.push_str("</td>");
                        }
                    }
                    _ => {
                        let mut tmp = String::new();
                        html::push_html(&mut tmp, std::iter::once(Event::End(tag_end)));
                        body.push_str(&tmp);
                    }
                }
            }
            Event::Text(ref text) => {
                if let Some(ref mut alt) = image_alt {
                    alt.push_str(text);
                } else {
                    body.push_str(&xml_escape(text));
                }
            }
            Event::Code(ref code) => {
                if image_alt.is_some() {
                    continue;
                }
                body.push_str(&format!(
                    r#"<code style="padding: 0.25em 0.4em; margin: 0; font-size: 85%; background-color: rgba(175,184,193,0.2); border-radius: 6px; font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;">{}</code>"#,
                    xml_escape(code)
                ));
            }
            Event::Rule => {
                if image_alt.is_some() {
                    continue;
                }
                body.push_str(r#"<hr style="height: .25em; padding: 0; margin: 24px 0; background-color: #d0d7de; border: 0;" />"#);
            }
            Event::TaskListMarker(checked) => {
                if image_alt.is_some() {
                    continue;
                }
                if checked {
                    body.push_str(r#"<input type="checkbox" checked disabled style="margin-right: 0.3em; vertical-align: middle;" />"#);
                } else {
                    body.push_str(r#"<input type="checkbox" disabled style="margin-right: 0.3em; vertical-align: middle;" />"#);
                }
            }
            Event::Html(ref html_content) | Event::InlineHtml(ref html_content) => {
                if image_alt.is_some() {
                    continue;
                }
                let mut replaced = html_content.to_string();
                let highlight_col = if config.appearance.highlight_color.is_empty() {
                    "#fff2a8"
                } else {
                    &config.appearance.highlight_color
                };

                replaced = replaced.replace(
                    r#"class="spoiler""#,
                    r#"class="spoiler" style="background-color: #b5bac1; color: #b5bac1; border-radius: 3px; padding: 0 3px; cursor: pointer;" onclick="if (this.style.color === 'inherit') { this.style.backgroundColor = '#b5bac1'; this.style.color = '#b5bac1'; } else { this.style.backgroundColor = 'rgba(79,84,92,0.3)'; this.style.color = 'inherit'; }""#
                );
                replaced = replaced.replace(
                    r#"class="footnote""#,
                    r#"class="footnote" style="font-size: 0.75em; background: #e0e0e0; color: #333; border-radius: 3px; padding: 0 3px; margin: 0 1px;""#
                );
                replaced = replaced.replace(
                    r#"class="named-quote""#,
                    r#"class="named-quote" style="border-left: 4px solid #888; margin: 0.5em 0; padding: 0.5em 1em; font-style: italic;""#
                );
                replaced = replaced.replace(
                    r#"class="quote-author""#,
                    r#"class="quote-author" style="display: block; font-style: normal; font-weight: bold; font-size: 0.9em; color: #666; margin-bottom: 0.25em;""#
                );
                replaced = replaced.replace(
                    "<details>",
                    r#"<details style="border: 1px solid #ccc; border-radius: 4px; padding: 0.5em 1em; margin: 0.5em 0;">"#
                );
                replaced = replaced.replace(
                    "<summary>",
                    r#"<summary style="cursor: pointer; font-weight: bold; list-style: none; padding: 0.25em 0;">"#
                );
                replaced = replaced.replace(
                    r#"class="error""#,
                    r#"class="error" style="text-decoration: underline wavy red;""#
                );
                replaced = replaced.replace(
                    "<mark>",
                    &format!(r#"<mark style="background-color: {};">"#, highlight_col)
                );
                replaced = replaced.replace("<strong>", r#"<strong style="font-weight: bold;">"#);
                replaced = replaced.replace("<em>", r#"<em style="font-style: italic;">"#);
                replaced = replaced.replace("<s>", r#"<s style="text-decoration: line-through;">"#);
                replaced = replaced.replace("<u>", r#"<u style="text-decoration: underline;">"#);
                replaced = replaced.replace("<sup>", r#"<sup style="vertical-align: super; font-size: smaller;">"#);
                replaced = replaced.replace("<sub>", r#"<sub style="vertical-align: sub; font-size: smaller;">"#);

                body.push_str(&replaced);
            }
            other => {
                if image_alt.is_some() {
                    continue;
                }
                let mut tmp = String::new();
                html::push_html(&mut tmp, std::iter::once(other));
                body.push_str(&tmp);
            }
        }
    }

    body
}

pub fn build_html_document_inline_styles(body: &str, title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color: #24292f; background-color: #ffffff; line-height: 1.6; max-width: 800px; margin: 40px auto; padding: 0 30px;">
{}
</body>
</html>"#,
        xml_escape(title),
        body
    )
}
