//! DOM tree utilities

use std::sync::Arc;
use crate::dom::{Document, Element, Node, NodeData};

/// DOM tree for efficient traversal and manipulation
pub struct DomTree {
    pub document: Document,
}

impl DomTree {
    /// Create a new DOM tree from a document
    pub fn new(document: Document) -> Self {
        Self { document }
    }

    /// Get the document element (html)
    pub fn document_element(&self) -> Option<Element> {
        self.document.document_element()
    }

    /// Get body element
    pub fn body(&self) -> Option<Element> {
        self.document.body()
    }

    /// Get head element
    pub fn head(&self) -> Option<Element> {
        self.document.head()
    }

    /// Get page title
    pub fn title(&self) -> Option<String> {
        self.document.title()
    }

    /// Get all script sources
    pub fn script_sources(&self) -> Vec<String> {
        self.document
            .scripts()
            .iter()
            .filter_map(|s| s.get_attr("src").cloned())
            .collect()
    }

    /// Get all stylesheet URLs
    pub fn stylesheet_urls(&self) -> Vec<String> {
        self.document
            .stylesheets()
            .iter()
            .filter_map(|s| s.get_attr("href").cloned())
            .collect()
    }

    /// Get all image sources
    pub fn image_sources(&self) -> Vec<String> {
        self.document
            .images()
            .iter()
            .filter_map(|img| {
                img.get_attr("src")
                    .or_else(|| img.get_attr("data-src"))
                    .cloned()
            })
            .collect()
    }

    /// Get all anchor hrefs
    pub fn links(&self) -> Vec<String> {
        self.document_element()
            .map(|e| {
                e.query_selector_all("a")
                    .iter()
                    .filter_map(|a| a.get_attr("href").cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get meta tags
    pub fn meta_tags(&self) -> Vec<(String, String)> {
        self.head()
            .map(|head| {
                head.query_selector_all("meta")
                    .iter()
                    .filter_map(|meta| {
                        let name = meta
                            .get_attr("name")
                            .or_else(|| meta.get_attr("property"))
                            .cloned()?;
                        let content = meta.get_attr("content").cloned()?;
                        Some((name, content))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if page has forms (for autofill warning)
    pub fn has_forms(&self) -> bool {
        self.body()
            .map(|b| !b.query_selector_all("form").is_empty())
            .unwrap_or(false)
    }

    /// Check if page has password fields
    pub fn has_password_fields(&self) -> bool {
        self.body()
            .map(|b| {
                b.query_selector_all("input")
                    .iter()
                    .any(|i| i.get_attr("type") == Some(&"password".to_string()))
            })
            .unwrap_or(false)
    }

    /// Get inline scripts (potential XSS vectors)
    pub fn inline_scripts(&self) -> Vec<String> {
        self.document
            .scripts()
            .iter()
            .filter(|s| s.get_attr("src").is_none())
            .map(|s| s.text_content())
            .filter(|t| !t.trim().is_empty())
            .collect()
    }

    /// Serialize to HTML string
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        if let Some(doc_elem) = self.document_element() {
            self.serialize_node(&doc_elem.node, &mut html);
        }
        html
    }

    fn serialize_node(&self, node: &Arc<Node>, output: &mut String) {
        match &node.data {
            NodeData::Element(elem) => {
                output.push('<');
                output.push_str(&elem.tag_name);
                
                for (name, value) in &elem.attributes {
                    output.push(' ');
                    output.push_str(name);
                    output.push_str("=\"");
                    output.push_str(&html_escape(value));
                    output.push('"');
                }
                
                if elem.is_void {
                    output.push_str(" />");
                } else {
                    output.push('>');
                    
                    for child in node.children.read().iter() {
                        self.serialize_node(child, output);
                    }
                    
                    output.push_str("</");
                    output.push_str(&elem.tag_name);
                    output.push('>');
                }
            }
            NodeData::Text(text) => {
                output.push_str(&html_escape(text));
            }
            NodeData::Comment(text) => {
                output.push_str("<!--");
                output.push_str(text);
                output.push_str("-->");
            }
            NodeData::DocType(dt) => {
                output.push_str("<!DOCTYPE ");
                output.push_str(&dt.name);
                output.push('>');
            }
            _ => {}
        }
    }
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
