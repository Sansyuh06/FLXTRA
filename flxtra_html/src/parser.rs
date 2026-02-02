//! HTML5 parser implementation

use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use markup5ever_rcdom::{Handle, NodeData as RcNodeData, RcDom};
use std::sync::Arc;
use tracing::debug;

use crate::dom::{Document, ElementData, Node, NodeData, NodeId, NodeType};

/// HTML5 parser
pub struct HtmlParser {
    opts: ParseOpts,
}

impl HtmlParser {
    /// Create a new HTML parser
    pub fn new() -> Self {
        Self {
            opts: ParseOpts {
                tree_builder: TreeBuilderOpts {
                    drop_doctype: false,
                    scripting_enabled: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    /// Parse HTML string into a Document
    pub fn parse(&self, html: &str) -> Document {
        debug!("Parsing HTML ({} bytes)", html.len());

        let dom = parse_document(RcDom::default(), self.opts.clone())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .expect("UTF-8 parsing should not fail");

        let doc = Document::new();
        self.convert_tree(&dom.document, &doc);
        doc
    }

    /// Parse HTML fragment
    pub fn parse_fragment(&self, html: &str) -> Document {
        // Wrap in html/body for fragment parsing
        let wrapped = format!("<!DOCTYPE html><html><body>{}</body></html>", html);
        self.parse(&wrapped)
    }

    /// Convert html5ever tree to our DOM
    fn convert_tree(&self, handle: &Handle, doc: &Document) {
        let node = self.convert_node(handle, doc);
        if let Some(n) = node {
            doc.root.children.write().push(n);
        }
    }

    fn convert_node(&self, handle: &Handle, doc: &Document) -> Option<Arc<Node>> {
        let node_data = &handle.data;

        let (node_type, data) = match node_data {
            RcNodeData::Document => {
                // Skip document node, we already have root
                for child in handle.children.borrow().iter() {
                    if let Some(n) = self.convert_node(child, doc) {
                        doc.root.children.write().push(n);
                    }
                }
                return None;
            }

            RcNodeData::Doctype { name, .. } => (
                NodeType::DocumentType,
                NodeData::DocType(crate::dom::DocTypeData {
                    name: name.to_string(),
                    public_id: String::new(),
                    system_id: String::new(),
                }),
            ),

            RcNodeData::Text { contents } => {
                let text = contents.borrow().to_string();
                if text.trim().is_empty() && text.len() < 100 {
                    // Skip pure whitespace (but keep significant whitespace)
                    return None;
                }
                (NodeType::Text, NodeData::Text(text))
            }

            RcNodeData::Comment { contents } => (
                NodeType::Comment,
                NodeData::Comment(contents.to_string()),
            ),

            RcNodeData::Element { name, attrs, .. } => {
                let tag = name.local.to_string();
                let mut elem_data = ElementData::new(&tag);

                // Copy attributes
                for attr in attrs.borrow().iter() {
                    elem_data
                        .attributes
                        .insert(attr.name.local.to_string(), attr.value.to_string());
                }

                (NodeType::Element, NodeData::Element(elem_data))
            }

            RcNodeData::ProcessingInstruction { target, contents } => (
                NodeType::ProcessingInstruction,
                NodeData::ProcessingInstruction {
                    target: target.to_string(),
                    data: contents.to_string(),
                },
            ),
        };

        let node = Arc::new(Node {
            id: doc.next_id(),
            node_type,
            parent: parking_lot::RwLock::new(None),
            children: parking_lot::RwLock::new(Vec::new()),
            data,
        });

        // Process children for element nodes
        if matches!(node.data, NodeData::Element(_)) {
            for child in handle.children.borrow().iter() {
                if let Some(child_node) = self.convert_node(child, doc) {
                    *child_node.parent.write() = Some(Arc::downgrade(&node));
                    node.children.write().push(child_node);
                }
            }
        }

        Some(node)
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let parser = HtmlParser::new();
        let doc = parser.parse("<html><head><title>Test</title></head><body><p>Hello</p></body></html>");
        
        assert!(doc.title().is_some());
        assert_eq!(doc.title().unwrap(), "Test");
    }

    #[test]
    fn test_query_selector() {
        let parser = HtmlParser::new();
        let doc = parser.parse(r#"
            <html>
                <body>
                    <div id="main" class="container">
                        <p class="intro">Hello</p>
                    </div>
                </body>
            </html>
        "#);

        let body = doc.body().unwrap();
        
        assert!(body.query_selector("#main").is_some());
        assert!(body.query_selector(".container").is_some());
        assert!(body.query_selector("p").is_some());
    }
}
