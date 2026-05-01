use sfmde::config::Config;
use sfmde::parser::render_to_html;

#[test]
fn test_debug_render() {
    let config = Config::default();
    let text = "_testing_\n\n*testing*";
    let html = render_to_html(text, &config);
    println!("{}", html);
    assert!(html.contains("<em>testing</em>"));
}
