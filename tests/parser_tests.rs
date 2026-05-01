use sfmde::config::Config;
use sfmde::parser::{render_to_html, xml_escape, build_html_document};

fn render(text: &str) -> String {
    render_to_html(text, &Config::default())
}

fn render_with<F: FnOnce(&mut Config)>(text: &str, f: F) -> String {
    let mut config = Config::default();
    f(&mut config);
    render_to_html(text, &config)
}

// ── xml_escape ────────────────────────────────────────────────────────────────

#[test]
fn test_xml_escape_ampersand() {
    assert_eq!(xml_escape("5 & 10"), "5 &amp; 10");
}

#[test]
fn test_xml_escape_less_than() {
    assert_eq!(xml_escape("a < b"), "a &lt; b");
}

#[test]
fn test_xml_escape_greater_than() {
    assert_eq!(xml_escape("a > b"), "a &gt; b");
}

#[test]
fn test_xml_escape_combined() {
    assert_eq!(xml_escape("<b>bold & strong</b>"), "&lt;b&gt;bold &amp; strong&lt;/b&gt;");
}

#[test]
fn test_xml_escape_no_special_chars() {
    assert_eq!(xml_escape("hello world"), "hello world");
}

// ── build_html_document ───────────────────────────────────────────────────────

#[test]
fn test_build_html_document_structure() {
    let doc = build_html_document("<p>hello</p>", "body { color: red; }", 0);
    assert!(doc.contains("<!DOCTYPE html>"), "got: {}", doc);
    assert!(doc.contains("<meta charset=\"utf-8\">"), "got: {}", doc);
    assert!(doc.contains("body { color: red; }"), "got: {}", doc);
    assert!(doc.contains("<p>hello</p>"), "got: {}", doc);
}

#[test]
fn test_build_html_document_error_class() {
    let doc = build_html_document("", "", 0);
    assert!(doc.contains(".error"), "got: {}", doc);
}

// ── Headings ──────────────────────────────────────────────────────────────────

#[test]
fn test_heading_h1() {
    assert!(render("# Hello").contains("<h1>Hello</h1>"), "got: {}", render("# Hello"));
}

#[test]
fn test_heading_h2() {
    assert!(render("## Hello").contains("<h2>Hello</h2>"));
}

#[test]
fn test_heading_h3() {
    assert!(render("### Hello").contains("<h3>Hello</h3>"));
}

#[test]
fn test_heading_h4() {
    assert!(render("#### Hello").contains("<h4>Hello</h4>"));
}

#[test]
fn test_heading_h5() {
    assert!(render("##### Hello").contains("<h5>Hello</h5>"));
}

#[test]
fn test_heading_h6() {
    assert!(render("###### Hello").contains("<h6>Hello</h6>"));
}

// ── Standard block elements ───────────────────────────────────────────────────

#[test]
fn test_paragraph_wrapping() {
    let out = render("Hello world");
    assert!(out.contains("<p>"), "got: {}", out);
    assert!(out.contains("Hello world"), "got: {}", out);
    assert!(out.contains("</p>"), "got: {}", out);
}

#[test]
fn test_two_paragraphs() {
    let out = render("first\n\nsecond");
    assert!(out.contains("first"), "got: {}", out);
    assert!(out.contains("second"), "got: {}", out);
}

#[test]
fn test_empty_input() {
    assert_eq!(render(""), "");
}

#[test]
fn test_horizontal_rule() {
    let out = render("---");
    assert!(out.contains("<hr"), "got: {}", out);
}

#[test]
fn test_blockquote() {
    let out = render("> quoted");
    assert!(out.contains("<blockquote>"), "got: {}", out);
    assert!(out.contains("quoted"), "got: {}", out);
    assert!(out.contains("</blockquote>"), "got: {}", out);
}

// ── Standard inline formatting (pulldown-cmark) ───────────────────────────────

#[test]
fn test_bold_standard() {
    assert!(render("**bold**").contains("<strong>bold</strong>"));
}

#[test]
fn test_italic_standard() {
    assert!(render("*italic*").contains("<em>italic</em>"));
}

#[test]
fn test_strikethrough_standard() {
    let out = render("~~strike~~");
    // SMD renders strikethrough as <s>
    assert!(out.contains("<s>strike</s>"), "got: {}", out);
}

#[test]
fn test_inline_code_standard() {
    let out = render("`code`");
    assert!(out.contains("<code>code</code>"), "got: {}", out);
}

#[test]
fn test_link_standard() {
    let out = render("[click](https://example.com)");
    assert!(out.contains("<a href=\"https://example.com\">click</a>"), "got: {}", out);
}

// ── Code blocks ───────────────────────────────────────────────────────────────

#[test]
fn test_code_block_output() {
    let out = render("```\nhello\n```");
    assert!(out.contains("<pre>"), "got: {}", out);
    assert!(out.contains("<code>"), "got: {}", out);
    assert!(out.contains("hello"), "got: {}", out);
}

#[test]
fn test_code_block_with_lang() {
    let out = render("```rust\nfn main() {}\n```");
    assert!(out.contains("fn main() {}"), "got: {}", out);
}

#[test]
fn test_code_block_xml_escapes_content() {
    let out = render("```\n<html>\n```");
    assert!(out.contains("&lt;html&gt;"), "got: {}", out);
}

#[test]
fn test_code_block_suppresses_smd_formatting() {
    // Inside a code block, the SMD inline parser is skipped
    let out = render("```\n||not a spoiler||\n```");
    assert!(!out.contains(r#"class="spoiler""#), "got: {}", out);
    assert!(out.contains("||not a spoiler||"), "got: {}", out);
}

// ── Lists ─────────────────────────────────────────────────────────────────────

#[test]
fn test_unordered_list() {
    let out = render("- item");
    assert!(out.contains("<ul>"), "got: {}", out);
    assert!(out.contains("<li>"), "got: {}", out);
    assert!(out.contains("item"), "got: {}", out);
}

#[test]
fn test_ordered_list() {
    let out = render("1. first\n2. second");
    assert!(out.contains("<ol>"), "got: {}", out);
    assert!(out.contains("<li>"), "got: {}", out);
}

#[test]
fn test_task_list_unchecked() {
    let out = render("- [ ] todo");
    assert!(out.contains(r#"type="checkbox""#), "got: {}", out);
}

#[test]
fn test_task_list_checked() {
    let out = render("- [x] done");
    assert!(out.contains("checked"), "got: {}", out);
}

// ── XML escaping (end-to-end) ─────────────────────────────────────────────────

#[test]
fn test_xml_escape_ampersand_in_text() {
    assert!(render("5 & 10").contains("5 &amp; 10"));
}

#[test]
fn test_xml_escape_angle_brackets_in_text() {
    let out = render("a < b and c > d");
    assert!(out.contains("a &lt; b"), "got: {}", out);
    assert!(out.contains("c &gt; d"), "got: {}", out);
}

#[test]
fn test_xml_escape_in_inline_code() {
    let out = render("`<br/>`");
    assert!(out.contains("&lt;br/&gt;"), "got: {}", out);
}

#[test]
fn test_xml_escape_in_code_block() {
    let out = render("```\n<html>\n```");
    assert!(out.contains("&lt;html&gt;"), "got: {}", out);
}

// ── Custom SMD inline formatting ──────────────────────────────────────────────

#[test]
fn test_smd_spoiler() {
    let out = render("||hidden||");
    assert!(out.contains(r#"class="spoiler""#), "got: {}", out);
    assert!(out.contains("hidden"), "got: {}", out);
}

#[test]
fn test_smd_spoiler_closed_properly() {
    let out = render("||hidden||");
    assert!(out.contains(r#"<span class="spoiler">hidden</span>"#), "got: {}", out);
}

#[test]
fn test_smd_highlight() {
    let out = render("==highlighted==");
    assert!(out.contains("<mark>highlighted</mark>"), "got: {}", out);
}

#[test]
fn test_smd_nested_highlight() {
    let out = render("==outer ==inner== outer==");
    // The first == opens, the second closes. The third opens, the fourth closes.
    // Result: <mark>outer </mark>inner<mark> outer</mark>
    assert!(out.contains("<mark>outer </mark>inner<mark> outer</mark>"), "got: {}", out);
}

#[test]
fn test_smd_interleaved_tags() {
    let out = render("==highlight ||spoiler|| highlight==");
    assert!(out.contains("<mark>highlight <span class=\"spoiler\">spoiler</span> highlight</mark>"), "got: {}", out);
}

#[test]
fn test_smd_nested_different_tags() {
    let out = render("||spoiler ==highlight== spoiler||");
    assert!(out.contains("<span class=\"spoiler\">spoiler <mark>highlight</mark> spoiler</span>"), "got: {}", out);
}

#[test]
fn test_smd_highlight_outermost() {
    let out = render("==highlight **bold** highlight==");
    assert!(out.contains("<mark>highlight <strong>bold</strong> highlight</mark>"), "got: {}", out);
}

#[test]
fn test_smd_superscript() {
    let out = render("x^2^");
    assert!(out.contains("<sup>2</sup>"), "got: {}", out);
}

#[test]
fn test_smd_subscript() {
    let out = render("H~2~O");
    assert!(out.contains("<sub>2</sub>"), "got: {}", out);
}

#[test]
fn test_smd_font_color_open_and_close() {
    let out = render("color:red some text color:end");
    assert!(out.contains("<span style=\"color: red\">"), "got: {}", out);
    assert!(out.contains("</span>"), "got: {}", out);
}

#[test]
fn test_smd_font_color_stops_at_space() {
    let out = render("color:blue text color:end");
    assert!(out.contains("<span style=\"color: blue\">"), "got: {}", out);
}

// ── Unclosed SMD tags → error spans ──────────────────────────────────────────

#[test]
fn test_smd_unclosed_spoiler_becomes_error() {
    let out = render("||unclosed");
    assert!(out.contains(r#"class="error""#), "got: {}", out);
    assert!(!out.contains(r#"class="spoiler""#), "got: {}", out);
}

#[test]
fn test_smd_unclosed_highlight_becomes_error() {
    let out = render("==unclosed");
    assert!(out.contains(r#"class="error""#), "got: {}", out);
}

#[test]
fn test_smd_unclosed_error_contains_raw_text() {
    let out = render("||unclosed");
    assert!(out.contains("||unclosed"), "got: {}", out);
}

// ── Custom config ─────────────────────────────────────────────────────────────

#[test]
fn test_custom_highlight_symbol() {
    let out = render_with("!!bright!!", |c| {
        c.formatters.highlight.symbol = "!!".to_string();
    });
    assert!(out.contains("<mark>bright</mark>"), "got: {}", out);
}

#[test]
fn test_custom_superscript_symbol() {
    let out = render_with("x@@2@@", |c| {
        c.formatters.superscript.symbol = "@@".to_string();
    });
    assert!(out.contains("<sup>2</sup>"), "got: {}", out);
}

#[test]
fn test_custom_bold_symbol() {
    let out = render_with("+++bold+++", |c| {
        c.formatters.bold.symbol = "+++".to_string();
    });
    assert!(out.contains("<strong>bold</strong>"), "got: {}", out);
}

#[test]
fn test_custom_spoiler_symbol() {
    let out = render_with("<<hidden>>", |c| {
        c.formatters.spoiler.symbol = "<<".to_string();
    });
    // "<<" opened but ">>" is not the spoiler symbol, so it stays open → error
    assert!(!out.is_empty(), "got: {}", out);
}

// ── Inline code suppresses SMD formatting ─────────────────────────────────────

#[test]
fn test_inline_code_suppresses_smd_formatting() {
    // pulldown-cmark emits inline code as Event::Code, not Event::Text,
    // so parse_smd_inlines_to_html is never called on it.
    let out = render("`||not a spoiler||`");
    assert!(!out.contains(r#"class="spoiler""#), "got: {}", out);
    assert!(out.contains("||not a spoiler||"), "got: {}", out);
}

// ── Combined formatting ───────────────────────────────────────────────────────

#[test]
fn test_bold_and_italic_combined() {
    let out = render("**bold** and *italic*");
    assert!(out.contains("<strong>bold</strong>"), "got: {}", out);
    assert!(out.contains("<em>italic</em>"), "got: {}", out);
}

#[test]
fn test_smd_and_markdown_combined() {
    let out = render("**bold** and ||spoiler||");
    assert!(out.contains("<strong>bold</strong>"), "got: {}", out);
    assert!(out.contains(r#"class="spoiler""#), "got: {}", out);
}
