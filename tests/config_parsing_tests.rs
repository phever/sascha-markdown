use sfmde::config::{Config, FormatterEntry, load_config, save_local_config};
use sfmde::parser::render_to_html;
use std::path::{Path, PathBuf};
use std::fs;

// A helper structure containing the set of symbols we'll test for a given config.
struct ExpectedTokens {
    italics: &'static str,
    bold: &'static str,
    underscore: &'static str,
    strikethrough: &'static str,
    spoiler: &'static str,
    highlight: &'static str,
    superscript: &'static str,
    subscript: &'static str,
    footnote: &'static str,
    font_color: &'static str,
    font_size_change: &'static str,
    named_quote: &'static str,
    collapse: &'static str,
    align_left: &'static str,
    align_right: &'static str,
    align_center: &'static str,
    align_justify: &'static str,
    emoji_prefix: &'static str,
}

// Function to generate the 5 configurations and write them to disk.
fn setup_test_directories() -> PathBuf {
    let base_dir = Path::new("target").join("test_smd_configs");
    let _ = fs::remove_dir_all(&base_dir); // clean up any previous runs
    fs::create_dir_all(&base_dir).unwrap();
    base_dir
}

fn write_config_to_dir(base_dir: &Path, name: &str, mut modifier: impl FnMut(&mut Config)) -> PathBuf {
    let dir = base_dir.join(name);
    fs::create_dir_all(&dir).unwrap();
    let mut config = Config::default();
    modifier(&mut config);
    save_local_config(&config, &dir).unwrap();
    dir
}

// Config 1: Standard (Default) Mappings
fn make_std_config(c: &mut Config) {
    // Already matches defaults
    c.formatters.italics = FormatterEntry::new("*", true, "icon");
    c.formatters.bold = FormatterEntry::new("**", true, "icon");
    c.formatters.underscore = FormatterEntry::new("_", true, "icon");
    c.formatters.strikethrough = FormatterEntry::new("~~", true, "icon");
    c.formatters.spoiler = FormatterEntry::new("||", true, "icon");
    c.formatters.highlight = FormatterEntry::new("==", true, "icon");
    c.formatters.superscript = FormatterEntry::new("^", true, "icon");
    c.formatters.subscript = FormatterEntry::new("~", true, "icon");
    c.formatters.footnote = FormatterEntry::new("[^]", true, "icon");
    c.formatters.font_color = FormatterEntry::new("color:", true, "icon");
    c.formatters.font_size_change = FormatterEntry::new("size:", true, "icon");
    c.formatters.named_quote = FormatterEntry::new("quote:", true, "icon");
    c.formatters.collapse = FormatterEntry::new("+++", true, "icon");
    c.formatters.align_left = FormatterEntry::new("[:left]", true, "icon");
    c.formatters.align_right = FormatterEntry::new("[:right]", true, "icon");
    c.formatters.align_center = FormatterEntry::new("[:center]", true, "icon");
    c.formatters.align_justify = FormatterEntry::new("[:justify]", true, "icon");
    c.formatters.emoji_prefix = FormatterEntry::new(":", true, "icon");
}

fn get_std_tokens() -> ExpectedTokens {
    ExpectedTokens {
        italics: "*",
        bold: "**",
        underscore: "_",
        strikethrough: "~~",
        spoiler: "||",
        highlight: "==",
        superscript: "^",
        subscript: "~",
        footnote: "[^]",
        font_color: "color:",
        font_size_change: "size:",
        named_quote: "quote:",
        collapse: "+++",
        align_left: "[:left]",
        align_right: "[:right]",
        align_center: "[:center]",
        align_justify: "[:justify]",
        emoji_prefix: ":",
    }
}

// Config 2: Alternative Custom Symbols
fn make_custom_config(c: &mut Config) {
    c.formatters.italics = FormatterEntry::new("!", true, "icon");
    c.formatters.bold = FormatterEntry::new("!!", true, "icon");
    c.formatters.underscore = FormatterEntry::new("__", true, "icon");
    c.formatters.strikethrough = FormatterEntry::new("--", true, "icon");
    c.formatters.spoiler = FormatterEntry::new("::", true, "icon");
    c.formatters.highlight = FormatterEntry::new("^^", true, "icon");
    c.formatters.superscript = FormatterEntry::new("_^_", true, "icon");
    c.formatters.subscript = FormatterEntry::new("_v_", true, "icon");
    c.formatters.footnote = FormatterEntry::new("[fn]", true, "icon");
    c.formatters.font_color = FormatterEntry::new("c:", true, "icon");
    c.formatters.font_size_change = FormatterEntry::new("s:", true, "icon");
    c.formatters.named_quote = FormatterEntry::new("q:", true, "icon");
    c.formatters.collapse = FormatterEntry::new(">>>", true, "icon");
    c.formatters.align_left = FormatterEntry::new("[<]", true, "icon");
    c.formatters.align_right = FormatterEntry::new("[>]", true, "icon");
    c.formatters.align_center = FormatterEntry::new("[=]", true, "icon");
    c.formatters.align_justify = FormatterEntry::new("[~]", true, "icon");
    c.formatters.emoji_prefix = FormatterEntry::new(".", true, "icon");
}

fn get_custom_tokens() -> ExpectedTokens {
    ExpectedTokens {
        italics: "!",
        bold: "!!",
        underscore: "__",
        strikethrough: "--",
        spoiler: "::",
        highlight: "^^",
        superscript: "_^_",
        subscript: "_v_",
        footnote: "[fn]",
        font_color: "c:",
        font_size_change: "s:",
        named_quote: "q:",
        collapse: ">>>",
        align_left: "[<]",
        align_right: "[>]",
        align_center: "[=]",
        align_justify: "[~]",
        emoji_prefix: ".",
    }
}

// Config 3: Overlapping & Single-character Symbols
fn make_overlap_config(c: &mut Config) {
    c.formatters.italics = FormatterEntry::new("/", true, "icon");
    c.formatters.bold = FormatterEntry::new("//", true, "icon");
    c.formatters.underscore = FormatterEntry::new("_", true, "icon");
    c.formatters.strikethrough = FormatterEntry::new("-", true, "icon");
    c.formatters.spoiler = FormatterEntry::new("|", true, "icon");
    c.formatters.highlight = FormatterEntry::new("%", true, "icon");
    c.formatters.superscript = FormatterEntry::new("+", true, "icon");
    c.formatters.subscript = FormatterEntry::new("=", true, "icon");
    c.formatters.footnote = FormatterEntry::new("^", true, "icon");
    c.formatters.font_color = FormatterEntry::new("col:", true, "icon");
    c.formatters.font_size_change = FormatterEntry::new("sz:", true, "icon");
    c.formatters.named_quote = FormatterEntry::new("n:", true, "icon");
    c.formatters.collapse = FormatterEntry::new("v", true, "icon");
    c.formatters.align_left = FormatterEntry::new("L", true, "icon");
    c.formatters.align_right = FormatterEntry::new("R", true, "icon");
    c.formatters.align_center = FormatterEntry::new("C", true, "icon");
    c.formatters.align_justify = FormatterEntry::new("J", true, "icon");
    c.formatters.emoji_prefix = FormatterEntry::new("@", true, "icon");
}

fn get_overlap_tokens() -> ExpectedTokens {
    ExpectedTokens {
        italics: "/",
        bold: "//",
        underscore: "_",
        strikethrough: "-",
        spoiler: "|",
        highlight: "%",
        superscript: "+",
        subscript: "=",
        footnote: "^",
        font_color: "col:",
        font_size_change: "sz:",
        named_quote: "n:",
        collapse: "v",
        align_left: "L",
        align_right: "R",
        align_center: "C",
        align_justify: "J",
        emoji_prefix: "@",
    }
}

// Config 4: Verbose Word-like / HTML-like Symbols
fn make_verbose_config(c: &mut Config) {
    c.formatters.italics = FormatterEntry::new("[i]", true, "icon");
    c.formatters.bold = FormatterEntry::new("[b]", true, "icon");
    c.formatters.underscore = FormatterEntry::new("[u]", true, "icon");
    c.formatters.strikethrough = FormatterEntry::new("[s]", true, "icon");
    c.formatters.spoiler = FormatterEntry::new("[spoiler]", true, "icon");
    c.formatters.highlight = FormatterEntry::new("[mark]", true, "icon");
    c.formatters.superscript = FormatterEntry::new("[sup]", true, "icon");
    c.formatters.subscript = FormatterEntry::new("[sub]", true, "icon");
    c.formatters.footnote = FormatterEntry::new("[foot]", true, "icon");
    c.formatters.font_color = FormatterEntry::new("[color]", true, "icon");
    c.formatters.font_size_change = FormatterEntry::new("[size]", true, "icon");
    c.formatters.named_quote = FormatterEntry::new("[blockquote]", true, "icon");
    c.formatters.collapse = FormatterEntry::new("[details]", true, "icon");
    c.formatters.align_left = FormatterEntry::new("[align-left]", true, "icon");
    c.formatters.align_right = FormatterEntry::new("[align-right]", true, "icon");
    c.formatters.align_center = FormatterEntry::new("[align-center]", true, "icon");
    c.formatters.align_justify = FormatterEntry::new("[align-justify]", true, "icon");
    c.formatters.emoji_prefix = FormatterEntry::new("[emoji]", true, "icon");
}

fn get_verbose_tokens() -> ExpectedTokens {
    ExpectedTokens {
        italics: "[i]",
        bold: "[b]",
        underscore: "[u]",
        strikethrough: "[s]",
        spoiler: "[spoiler]",
        highlight: "[mark]",
        superscript: "[sup]",
        subscript: "[sub]",
        footnote: "[foot]",
        font_color: "[color]",
        font_size_change: "[size]",
        named_quote: "[blockquote]",
        collapse: "[details]",
        align_left: "[align-left]",
        align_right: "[align-right]",
        align_center: "[align-center]",
        align_justify: "[align-justify]",
        emoji_prefix: "[emoji]",
    }
}

// Config 5: Weird Unicode / Emoji-based Symbols
fn make_weird_config(c: &mut Config) {
    c.formatters.italics = FormatterEntry::new("*~", true, "icon");
    c.formatters.bold = FormatterEntry::new("~*", true, "icon");
    c.formatters.underscore = FormatterEntry::new("✦", true, "icon");
    c.formatters.strikethrough = FormatterEntry::new("⁓", true, "icon");
    c.formatters.spoiler = FormatterEntry::new("§§", true, "icon");
    c.formatters.highlight = FormatterEntry::new("°°", true, "icon");
    c.formatters.superscript = FormatterEntry::new("↑", true, "icon");
    c.formatters.subscript = FormatterEntry::new("↓", true, "icon");
    c.formatters.footnote = FormatterEntry::new("※", true, "icon");
    c.formatters.font_color = FormatterEntry::new("🎨:", true, "icon");
    c.formatters.font_size_change = FormatterEntry::new("📏:", true, "icon");
    c.formatters.named_quote = FormatterEntry::new("🗣️:", true, "icon");
    c.formatters.collapse = FormatterEntry::new("📂:", true, "icon");
    c.formatters.align_left = FormatterEntry::new("👈:", true, "icon");
    c.formatters.align_right = FormatterEntry::new("👉:", true, "icon");
    c.formatters.align_center = FormatterEntry::new("👆:", true, "icon");
    c.formatters.align_justify = FormatterEntry::new("👇:", true, "icon");
    c.formatters.emoji_prefix = FormatterEntry::new("😀:", true, "icon");
}

fn get_weird_tokens() -> ExpectedTokens {
    ExpectedTokens {
        italics: "*~",
        bold: "~*",
        underscore: "✦",
        strikethrough: "⁓",
        spoiler: "§§",
        highlight: "°°",
        superscript: "↑",
        subscript: "↓",
        footnote: "※",
        font_color: "🎨:",
        font_size_change: "📏:",
        named_quote: "🗣️:",
        collapse: "📂:",
        align_left: "👈:",
        align_right: "👉:",
        align_center: "👆:",
        align_justify: "👇:",
        emoji_prefix: "😀:",
    }
}

// CORE EXHAUSTIVE PARSING TESTING FUNCTION
fn run_assertions_for_config(config: &Config, tokens: &ExpectedTokens, label: &str) {
    println!("Testing configuration: {}", label);

    // 1. Italics
    let input = format!("{}italic{}", tokens.italics, tokens.italics);
    let output = render_to_html(&input, config);
    assert!(
        output.contains("<em>italic</em>"),
        "Failed italics in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 2. Bold
    let input = format!("{}bold{}", tokens.bold, tokens.bold);
    let output = render_to_html(&input, config);
    assert!(
        output.contains("<strong>bold</strong>"),
        "Failed bold in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 3. Underscore
    let input = format!("{}underline{}", tokens.underscore, tokens.underscore);
    let output = render_to_html(&input, config);
    assert!(
        output.contains("<u>underline</u>"),
        "Failed underscore in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 4. Strikethrough
    let input = format!("{}strike{}", tokens.strikethrough, tokens.strikethrough);
    let output = render_to_html(&input, config);
    assert!(
        output.contains("<s>strike</s>"),
        "Failed strikethrough in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 5. Spoiler
    let input = format!("{}spoiler{}", tokens.spoiler, tokens.spoiler);
    let output = render_to_html(&input, config);
    assert!(
        output.contains(r#"<span class="spoiler">spoiler</span>"#),
        "Failed spoiler in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 6. Highlight
    let input = format!("{}highlight{}", tokens.highlight, tokens.highlight);
    let output = render_to_html(&input, config);
    assert!(
        output.contains("<mark>highlight</mark>"),
        "Failed highlight in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 7. Superscript
    let input = format!("x{}2{}", tokens.superscript, tokens.superscript);
    let output = render_to_html(&input, config);
    assert!(
        output.contains("<sup>2</sup>"),
        "Failed superscript in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 8. Subscript
    let input = format!("y{}sub{}", tokens.subscript, tokens.subscript);
    let output = render_to_html(&input, config);
    assert!(
        output.contains("<sub>sub</sub>"),
        "Failed subscript in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 9. Footnote
    let input = format!("text{}1{}", tokens.footnote, tokens.footnote);
    let output = render_to_html(&input, config);
    assert!(
        output.contains(r#"<sup class="footnote">1</sup>"#),
        "Failed footnote in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 10. Font Color
    let input = format!("{}red special text{}end", tokens.font_color, tokens.font_color);
    let output = render_to_html(&input, config);
    assert!(
        output.contains(r#"<span style="color: red">"#) && output.contains("</span>"),
        "Failed font_color in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 11. Font Size
    let input = format!("{}18 size text{}end", tokens.font_size_change, tokens.font_size_change);
    let output = render_to_html(&input, config);
    assert!(
        output.contains(r#"<span style="font-size: 18pt">"#) && output.contains("</span>"),
        "Failed font_size_change in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 12. Named Quote
    let input = format!("{}Sascha hello{}end", tokens.named_quote, tokens.named_quote);
    let output = render_to_html(&input, config);
    assert!(
        output.contains(r#"<blockquote class="named-quote"><cite class="quote-author">Sascha</cite> hello"#),
        "Failed named_quote in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 13. Collapse
    let input = format!("{}DetailsSummary expand content{}end", tokens.collapse, tokens.collapse);
    let output = render_to_html(&input, config);
    assert!(
        output.contains("<details><summary>DetailsSummary</summary>") && output.contains("</details>"),
        "Failed collapse in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 14. Align Left
    let input = format!("{}aligned left{}", tokens.align_left, tokens.align_left);
    let output = render_to_html(&input, config);
    assert!(
        output.contains(r#"<span style="display:block;text-align:left">aligned left</span>"#),
        "Failed align_left in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 15. Align Right
    let input = format!("{}aligned right{}", tokens.align_right, tokens.align_right);
    let output = render_to_html(&input, config);
    assert!(
        output.contains(r#"<span style="display:block;text-align:right">aligned right</span>"#),
        "Failed align_right in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 16. Align Center
    let input = format!("{}aligned center{}", tokens.align_center, tokens.align_center);
    let output = render_to_html(&input, config);
    assert!(
        output.contains(r#"<span style="display:block;text-align:center">aligned center</span>"#),
        "Failed align_center in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 17. Align Justify
    let input = format!("{}aligned justify{}", tokens.align_justify, tokens.align_justify);
    let output = render_to_html(&input, config);
    assert!(
        output.contains(r#"<span style="display:block;text-align:justify">aligned justify</span>"#),
        "Failed align_justify in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 18. Emoji prefix
    let input = format!("{}smile{}", tokens.emoji_prefix, tokens.emoji_prefix);
    let output = render_to_html(&input, config);
    assert!(
        output.contains("🙂"),
        "Failed emoji_prefix in {}. Input: '{}', Output: '{}'",
        label, input, output
    );

    // 19. Nesting tags: Bold inside Spoiler
    // Note: outer spoiler closes when tokens.spoiler matched.
    // Let's make nesting correct: outer spoiler, inner bold
    let input_nest = format!("{}outer {}bold{} outer{}", tokens.spoiler, tokens.bold, tokens.bold, tokens.spoiler);
    let output_nest = render_to_html(&input_nest, config);
    assert!(
        output_nest.contains(r#"class="spoiler""#) && output_nest.contains("<strong>bold</strong>"),
        "Failed nested styles in {}. Input: '{}', Output: '{}'",
        label, input_nest, output_nest
    );

    // 20. Unclosed tags becomes error span
    let input_unclosed = format!("{}unclosed tag text", tokens.bold);
    let output_unclosed = render_to_html(&input_unclosed, config);
    assert!(
        output_unclosed.contains(r#"class="error""#) && output_unclosed.contains("unclosed tag text"),
        "Failed unclosed tags in {}. Input: '{}', Output: '{}'",
        label, input_unclosed, output_unclosed
    );

    // 21. Suppressed formatting inside inline code
    let input_suppressed = format!("`{}suppressed{}`", tokens.bold, tokens.bold);
    let output_suppressed = render_to_html(&input_suppressed, config);
    assert!(
        !output_suppressed.contains("<strong>"),
        "Failed suppressed style in code in {}. Input: '{}', Output: '{}'",
        label, input_suppressed, output_suppressed
    );
}

#[test]
fn test_all_five_configurations_and_symbols() {
    let base_dir = setup_test_directories();

    // 1. Write the 5 configurations to files
    let std_dir = write_config_to_dir(&base_dir, "std", make_std_config);
    let custom_dir = write_config_to_dir(&base_dir, "custom", make_custom_config);
    let overlap_dir = write_config_to_dir(&base_dir, "overlap", make_overlap_config);
    let verbose_dir = write_config_to_dir(&base_dir, "verbose", make_verbose_config);
    let weird_dir = write_config_to_dir(&base_dir, "weird", make_weird_config);

    // 2. Load them back using `load_config` to verify deserialization works perfectly!
    let config_std = load_config(&std_dir).expect("Load standard config");
    let config_custom = load_config(&custom_dir).expect("Load custom config");
    let config_overlap = load_config(&overlap_dir).expect("Load overlapping config");
    let config_verbose = load_config(&verbose_dir).expect("Load verbose config");
    let config_weird = load_config(&weird_dir).expect("Load weird config");

    // 3. Perform the massive parser checks on each loaded configuration
    run_assertions_for_config(&config_std, &get_std_tokens(), "Standard/Default Config");
    run_assertions_for_config(&config_custom, &get_custom_tokens(), "Custom Config");
    run_assertions_for_config(&config_overlap, &get_overlap_tokens(), "Overlapping/Short Config");
    run_assertions_for_config(&config_verbose, &get_verbose_tokens(), "Verbose HTML-like Config");
    run_assertions_for_config(&config_weird, &get_weird_tokens(), "Weird Unicode/Emoji Config");
}
