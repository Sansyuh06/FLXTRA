//! Aegis HTML Parser
//!
//! HTML5-compliant parser with:
//! - Streaming parsing support
//! - DOM tree construction
//! - Script extraction for security analysis
//! - Form detection for autofill blocking

pub mod dom;
pub mod parser;
pub mod tree;

pub use dom::{Document, Element, Node, NodeType};
pub use parser::HtmlParser;
pub use tree::DomTree;
