pub mod emoji;
pub mod preprocessor;
pub mod renderer;

// Re-export the public API so that the rest of the application
// compiles without requiring import modifications.
#[allow(unused_imports)]
pub use preprocessor::xml_escape;
pub use renderer::{
    build_html_document, build_html_document_inline_styles, render_to_html, render_to_html_inline,
};
