//! DOM node types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use parking_lot::RwLock;

/// Unique node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

/// Node type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Document,
    Element,
    Text,
    Comment,
    DocumentType,
    ProcessingInstruction,
}

/// DOM Node
#[derive(Debug)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub parent: RwLock<Option<Weak<Node>>>,
    pub children: RwLock<Vec<Arc<Node>>>,
    pub data: NodeData,
}

/// Node-specific data
#[derive(Debug, Clone)]
pub enum NodeData {
    Document,
    Element(ElementData),
    Text(String),
    Comment(String),
    DocType(DocTypeData),
    ProcessingInstruction { target: String, data: String },
}

/// Element-specific data
#[derive(Debug, Clone)]
pub struct ElementData {
    pub tag_name: String,
    pub namespace: Option<String>,
    pub attributes: HashMap<String, String>,
    pub is_void: bool,
}

impl ElementData {
    pub fn new(tag_name: &str) -> Self {
        let is_void = matches!(
            tag_name.to_lowercase().as_str(),
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" 
            | "link" | "meta" | "param" | "source" | "track" | "wbr"
        );

        Self {
            tag_name: tag_name.to_string(),
            namespace: None,
            attributes: HashMap::new(),
            is_void,
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    pub fn set_attr(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    pub fn remove_attr(&mut self, name: &str) {
        self.attributes.remove(name);
    }

    /// Get ID attribute
    pub fn id(&self) -> Option<&String> {
        self.get_attr("id")
    }

    /// Get class list
    pub fn class_list(&self) -> Vec<&str> {
        self.get_attr("class")
            .map(|c| c.split_whitespace().collect())
            .unwrap_or_default()
    }

    /// Check if element has a class
    pub fn has_class(&self, class: &str) -> bool {
        self.class_list().contains(&class)
    }
}

/// DOCTYPE data
#[derive(Debug, Clone)]
pub struct DocTypeData {
    pub name: String,
    pub public_id: String,
    pub system_id: String,
}

/// Element wrapper for convenient DOM manipulation
#[derive(Debug, Clone)]
pub struct Element {
    pub node: Arc<Node>,
}

impl Element {
    pub fn new(node: Arc<Node>) -> Option<Self> {
        if matches!(node.data, NodeData::Element(_)) {
            Some(Self { node })
        } else {
            None
        }
    }

    pub fn data(&self) -> &ElementData {
        match &self.node.data {
            NodeData::Element(data) => data,
            _ => panic!("Element wrapper on non-element node"),
        }
    }

    pub fn tag_name(&self) -> &str {
        &self.data().tag_name
    }

    pub fn get_attr(&self, name: &str) -> Option<&String> {
        self.data().get_attr(name)
    }

    pub fn children(&self) -> Vec<Arc<Node>> {
        self.node.children.read().clone()
    }

    pub fn child_elements(&self) -> Vec<Element> {
        self.children()
            .into_iter()
            .filter_map(|n| Element::new(n))
            .collect()
    }

    pub fn text_content(&self) -> String {
        self.collect_text(&self.node)
    }

    fn collect_text(&self, node: &Arc<Node>) -> String {
        let mut text = String::new();
        
        match &node.data {
            NodeData::Text(t) => text.push_str(t),
            _ => {
                for child in node.children.read().iter() {
                    text.push_str(&self.collect_text(child));
                }
            }
        }
        
        text
    }

    /// Query selector (simplified - supports tag, class, id)
    pub fn query_selector(&self, selector: &str) -> Option<Element> {
        self.query_selector_all(selector).into_iter().next()
    }

    /// Query selector all (simplified)
    pub fn query_selector_all(&self, selector: &str) -> Vec<Element> {
        let mut results = Vec::new();
        self.query_recursive(&self.node, selector, &mut results);
        results
    }

    fn query_recursive(&self, node: &Arc<Node>, selector: &str, results: &mut Vec<Element>) {
        if let Some(elem) = Element::new(node.clone()) {
            if self.matches_selector(&elem, selector) {
                results.push(elem.clone());
            }
            
            for child in elem.children() {
                self.query_recursive(&child, selector, results);
            }
        }
    }

    fn matches_selector(&self, elem: &Element, selector: &str) -> bool {
        let selector = selector.trim();
        
        // ID selector
        if selector.starts_with('#') {
            let id = &selector[1..];
            return elem.data().id() == Some(&id.to_string());
        }
        
        // Class selector
        if selector.starts_with('.') {
            let class = &selector[1..];
            return elem.data().has_class(class);
        }
        
        // Tag selector
        elem.tag_name().eq_ignore_ascii_case(selector)
    }
}

/// Document wrapper
#[derive(Debug)]
pub struct Document {
    pub root: Arc<Node>,
    next_id: std::sync::atomic::AtomicU64,
}

impl Document {
    pub fn new() -> Self {
        Self {
            root: Arc::new(Node {
                id: NodeId(0),
                node_type: NodeType::Document,
                parent: RwLock::new(None),
                children: RwLock::new(Vec::new()),
                data: NodeData::Document,
            }),
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    pub fn next_id(&self) -> NodeId {
        NodeId(self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }

    pub fn document_element(&self) -> Option<Element> {
        self.root
            .children
            .read()
            .iter()
            .find_map(|n| Element::new(n.clone()))
    }

    pub fn body(&self) -> Option<Element> {
        self.document_element()?
            .query_selector("body")
    }

    pub fn head(&self) -> Option<Element> {
        self.document_element()?
            .query_selector("head")
    }

    pub fn title(&self) -> Option<String> {
        self.head()?
            .query_selector("title")
            .map(|e| e.text_content())
    }

    /// Get all script elements
    pub fn scripts(&self) -> Vec<Element> {
        self.document_element()
            .map(|e| e.query_selector_all("script"))
            .unwrap_or_default()
    }

    /// Get all link elements
    pub fn stylesheets(&self) -> Vec<Element> {
        self.document_element()
            .map(|e| {
                e.query_selector_all("link")
                    .into_iter()
                    .filter(|l| l.get_attr("rel") == Some(&"stylesheet".to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all images
    pub fn images(&self) -> Vec<Element> {
        self.document_element()
            .map(|e| e.query_selector_all("img"))
            .unwrap_or_default()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
