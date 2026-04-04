//! HTML Parser
//! 
//! Responsible for:
//! - Parsing HTML documents into a DOM tree using html5ever
//! - Building semantic AST from HTML content
//! - Exposing types that feed into flxtra_css (styling) and flxtra_layout (layout engine)
//!
//! The DOM tree produced here is consumed by CSS resolution in flxtra_css.

pub struct HtmlParser;

impl HtmlParser {
    pub fn new() -> Self {
        Self
    }
}
