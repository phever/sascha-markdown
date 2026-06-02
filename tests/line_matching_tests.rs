use sfmde::config::Config;
use sfmde::parser::render_to_html;
use regex::Regex;

#[derive(Debug, PartialEq)]
struct MatchedLine {
    tag_name: String,
    line_number: u32,
}

fn get_line_mappings(text: &str) -> Vec<MatchedLine> {
    let html = render_to_html(text, &Config::default());
    let re = Regex::new(r#"<([a-zA-Z1-6]+)[^>]*data-src-line="(\d+)"[^>]*>"#).unwrap();
    re.captures_iter(&html)
        .map(|cap| MatchedLine {
            tag_name: cap[1].to_string(),
            line_number: cap[2].parse().unwrap(),
        })
        .collect()
}

#[test]
fn test_line_matching_simple_markdown() {
    let input = "\
# Heading

This is a paragraph.";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "h1".to_string(), line_number: 1 },
            MatchedLine { tag_name: "p".to_string(), line_number: 3 },
        ]
    );
}

#[test]
fn test_line_matching_multiline_paragraph() {
    let input = "\
This is a
multiline paragraph
spanning three lines.";
    let mappings = get_line_mappings(input);
    // The paragraph starts on line 1.
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
        ]
    );
}

#[test]
fn test_line_matching_lists() {
    let input = "\
1. First ordered
2. Second ordered

- First unordered
- Second unordered";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "ol".to_string(), line_number: 1 },
            MatchedLine { tag_name: "li".to_string(), line_number: 1 },
            MatchedLine { tag_name: "li".to_string(), line_number: 2 },
            MatchedLine { tag_name: "ul".to_string(), line_number: 4 },
            MatchedLine { tag_name: "li".to_string(), line_number: 4 },
            MatchedLine { tag_name: "li".to_string(), line_number: 5 },
        ]
    );
}

#[test]
fn test_line_matching_nested_lists() {
    let input = "\
- Item 1
  - Subitem 1.1
  - Subitem 1.2
- Item 2";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "ul".to_string(), line_number: 1 },
            MatchedLine { tag_name: "li".to_string(), line_number: 1 },
            MatchedLine { tag_name: "ul".to_string(), line_number: 2 },
            MatchedLine { tag_name: "li".to_string(), line_number: 2 },
            MatchedLine { tag_name: "li".to_string(), line_number: 3 },
            MatchedLine { tag_name: "li".to_string(), line_number: 4 },
        ]
    );
}

#[test]
fn test_line_matching_blockquotes() {
    let input = "\
> This is a quote.
> It spans two lines.";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "blockquote".to_string(), line_number: 1 },
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
        ]
    );
}

#[test]
fn test_line_matching_code_blocks() {
    let input = "\
```rust
fn main() {
    println!(\"Hello\");
}
```";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "pre".to_string(), line_number: 1 },
        ]
    );
}

#[test]
fn test_line_matching_horizontal_rule() {
    let input = "\
Line before

---

Line after";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
            MatchedLine { tag_name: "hr".to_string(), line_number: 3 },
            MatchedLine { tag_name: "p".to_string(), line_number: 5 },
        ]
    );
}

#[test]
fn test_line_matching_tables() {
    let input = "\
| Col 1 | Col 2 |
|---|---|
| Val 1 | Val 2 |";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "table".to_string(), line_number: 1 },
        ]
    );
}

#[test]
fn test_line_matching_unclosed_tags_errors() {
    let input = "\
||unclosed spoiler

==unclosed highlight

正常段落";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
            MatchedLine { tag_name: "p".to_string(), line_number: 3 },
            MatchedLine { tag_name: "p".to_string(), line_number: 5 },
        ]
    );
    
    // Also verify that the HTML output wraps the unclosed tags as errors
    let html = render_to_html(input, &Config::default());
    assert!(html.contains(r#"<span class="error">||unclosed spoiler"#));
    assert!(html.contains(r#"<span class="error">==unclosed highlight"#));
}

#[test]
fn test_line_matching_nested_unclosed_tags() {
    let input = "||outer **inner unclosed";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
        ]
    );
    let html = render_to_html(input, &Config::default());
    assert!(html.contains(r#"<span class="error">||outer <span class="error">**inner unclosed</span></span>"#));
}

#[test]
fn test_line_matching_crlf_endings() {
    let input = "# CRLF Heading\r\n\r\nCRLF Paragraph.\r\n";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "h1".to_string(), line_number: 1 },
            MatchedLine { tag_name: "p".to_string(), line_number: 3 },
        ]
    );
}

#[test]
fn test_line_matching_mixed_html_blocks() {
    let input = "\
<div>
  Some HTML block
</div>

# Heading after HTML";
    let mappings = get_line_mappings(input);
    // HTML block doesn't get data-src-line, but the Heading after it should have line 5.
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "h1".to_string(), line_number: 5 },
        ]
    );
}

#[test]
fn test_line_matching_unicode_and_emojis() {
    let input = "\
🙂 Hello :smile: world
and :slightly_smiling_face: emoji

Next block";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
            MatchedLine { tag_name: "p".to_string(), line_number: 4 },
        ]
    );
}

#[test]
fn test_line_matching_malformed_font_color_and_size() {
    let input = "\
color:red unclosed color

size:12 unclosed size

正常段落";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
            MatchedLine { tag_name: "p".to_string(), line_number: 3 },
            MatchedLine { tag_name: "p".to_string(), line_number: 5 },
        ]
    );
    let html = render_to_html(input, &Config::default());
    assert!(html.contains(r#"<span class="error">color:red"#));
    assert!(html.contains(r#"<span class="error">size:12"#));
}

#[test]
fn test_line_matching_complex_nested_styles() {
    let input = "\
**bold ||spoiler** spoiler close||

More text";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
            MatchedLine { tag_name: "p".to_string(), line_number: 3 },
        ]
    );
    let html = render_to_html(input, &Config::default());
    // Since bold closes first, ||spoiler is nested inside bold but unclosed at the time bold closes.
    // So ||spoiler should be converted to an error span, and then bold should close normally.
    assert!(html.contains(r#"<strong>bold <span class="error">||spoiler</span></strong>"#));
}

#[test]
fn test_line_matching_extremely_nested_unclosed() {
    let input = "\
||spoiler **bold ==highlight _underline ~subscript^superscript^ subscript end~ underline end_ highlight end== bold end** spoiler end||";
    let mappings = get_line_mappings(input);
    assert_eq!(
        mappings,
        vec![
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
        ]
    );
    
    let malformed_input = "\
||spoiler **bold ==highlight _underline ~subscript^superscript";
    let mappings_malformed = get_line_mappings(malformed_input);
    assert_eq!(
        mappings_malformed,
        vec![
            MatchedLine { tag_name: "p".to_string(), line_number: 1 },
        ]
    );
    let html = render_to_html(malformed_input, &Config::default());
    // All unclosed tags should be converted to error spans, nested correctly:
    assert!(html.contains("<span class=\"error\">||spoiler <span class=\"error\">**bold <span class=\"error\">==highlight <span class=\"error\">_underline <span class=\"error\">~subscript<span class=\"error\">^superscript</span></span></span></span></span></span>"));
}
