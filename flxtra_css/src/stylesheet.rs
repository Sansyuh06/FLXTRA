//! Stylesheet types

use crate::properties::Property;
use serde::{Deserialize, Serialize};

/// A complete stylesheet
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleSheet {
    pub rules: Vec<StyleRule>,
    pub url: Option<String>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_url(url: &str) -> Self {
        Self {
            rules: Vec::new(),
            url: Some(url.to_string()),
        }
    }

    pub fn add_rule(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }
}

/// A CSS rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleRule {
    Style(StyleDeclaration),
    Media(MediaRule),
    Import(String),
    FontFace(Vec<Property>),
}

/// A style declaration (selector + properties)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleDeclaration {
    pub selectors: Vec<Selector>,
    pub properties: Vec<Property>,
}

impl StyleDeclaration {
    pub fn new(selector: &str) -> Self {
        Self {
            selectors: vec![Selector::parse(selector)],
            properties: Vec::new(),
        }
    }

    pub fn add_property(&mut self, prop: Property) {
        self.properties.push(prop);
    }
}

/// Media query rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRule {
    pub query: String,
    pub rules: Vec<StyleDeclaration>,
}

/// CSS selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selector {
    pub raw: String,
    pub parts: Vec<SelectorPart>,
    pub specificity: Specificity,
}

impl Selector {
    pub fn parse(selector: &str) -> Self {
        let parts = parse_selector_parts(selector);
        let specificity = calculate_specificity(&parts);
        
        Self {
            raw: selector.to_string(),
            parts,
            specificity,
        }
    }

    /// Check if selector matches an element
    pub fn matches(&self, tag: &str, id: Option<&str>, classes: &[&str]) -> bool {
        for part in &self.parts {
            match part {
                SelectorPart::Universal => return true,
                SelectorPart::Tag(t) => {
                    if t.eq_ignore_ascii_case(tag) {
                        return true;
                    }
                }
                SelectorPart::Class(c) => {
                    if classes.contains(&c.as_str()) {
                        return true;
                    }
                }
                SelectorPart::Id(i) => {
                    if id == Some(i.as_str()) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

/// Selector part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectorPart {
    Universal,              // *
    Tag(String),           // div
    Class(String),         // .class
    Id(String),            // #id
    Attribute(String),     // [attr]
    PseudoClass(String),   // :hover
    PseudoElement(String), // ::before
    Combinator(Combinator),
}

/// Selector combinator
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Combinator {
    Descendant,    // space
    Child,         // >
    NextSibling,   // +
    Sibling,       // ~
}

/// Selector specificity (a, b, c)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Specificity {
    pub ids: u32,
    pub classes: u32,
    pub tags: u32,
}

impl Specificity {
    pub fn new(ids: u32, classes: u32, tags: u32) -> Self {
        Self { ids, classes, tags }
    }
}

fn parse_selector_parts(selector: &str) -> Vec<SelectorPart> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = selector.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '#' => {
                if !current.is_empty() {
                    parts.push(SelectorPart::Tag(current.clone()));
                    current.clear();
                }
                let id: String = chars.by_ref().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();
                if !id.is_empty() {
                    parts.push(SelectorPart::Id(id));
                }
            }
            '.' => {
                if !current.is_empty() {
                    parts.push(SelectorPart::Tag(current.clone()));
                    current.clear();
                }
                let class: String = chars.by_ref().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();
                if !class.is_empty() {
                    parts.push(SelectorPart::Class(class));
                }
            }
            '*' => {
                parts.push(SelectorPart::Universal);
            }
            ' ' | '>' | '+' | '~' => {
                if !current.is_empty() {
                    parts.push(SelectorPart::Tag(current.clone()));
                    current.clear();
                }
                let combinator = match c {
                    '>' => Combinator::Child,
                    '+' => Combinator::NextSibling,
                    '~' => Combinator::Sibling,
                    _ => Combinator::Descendant,
                };
                parts.push(SelectorPart::Combinator(combinator));
            }
            ':' => {
                if !current.is_empty() {
                    parts.push(SelectorPart::Tag(current.clone()));
                    current.clear();
                }
                let is_element = chars.peek() == Some(&':');
                if is_element {
                    chars.next();
                }
                let pseudo: String = chars.by_ref().take_while(|c| c.is_alphanumeric() || *c == '-').collect();
                if is_element {
                    parts.push(SelectorPart::PseudoElement(pseudo));
                } else {
                    parts.push(SelectorPart::PseudoClass(pseudo));
                }
            }
            '[' => {
                let attr: String = chars.by_ref().take_while(|c| *c != ']').collect();
                parts.push(SelectorPart::Attribute(attr));
            }
            _ if c.is_alphanumeric() || c == '_' || c == '-' => {
                current.push(c);
            }
            _ => {}
        }
    }

    if !current.is_empty() {
        parts.push(SelectorPart::Tag(current));
    }

    parts
}

fn calculate_specificity(parts: &[SelectorPart]) -> Specificity {
    let mut spec = Specificity::default();
    
    for part in parts {
        match part {
            SelectorPart::Id(_) => spec.ids += 1,
            SelectorPart::Class(_) | SelectorPart::Attribute(_) | SelectorPart::PseudoClass(_) => {
                spec.classes += 1
            }
            SelectorPart::Tag(_) | SelectorPart::PseudoElement(_) => spec.tags += 1,
            _ => {}
        }
    }
    
    spec
}
