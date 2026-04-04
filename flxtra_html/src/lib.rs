//! HTML Parser
//! 
//! Responsible for:
//! - Parsing HTML documents into a DOM tree
//! - Building semantic AST from HTML content
//! - Exposing types that feed into flxtra_css (styling) and flxtra_layout (layout engine)
//!
//! The DOM tree produced here is consumed by CSS resolution in flxtra_css.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Document,
    Element(String), // tag name
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct DomNode {
    pub node_type: NodeType,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Box<DomNode>>,
}

pub struct HtmlParser;

impl HtmlParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(html: &str) -> Result<DomNode, String> {
        let mut root = DomNode {
            node_type: NodeType::Document,
            attributes: HashMap::new(),
            children: Vec::new(),
        };

        parse_html_content(html, &mut root)?;
        Ok(root)
    }
}

fn parse_html_content(html: &str, parent: &mut DomNode) -> Result<(), String> {
    let mut chars = html.chars().peekable();
    let mut current_text = String::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            '<' => {
                if !current_text.is_empty() {
                    parent.children.push(Box::new(DomNode {
                        node_type: NodeType::Text(current_text.clone()),
                        attributes: HashMap::new(),
                        children: Vec::new(),
                    }));
                    current_text.clear();
                }

                chars.next();
                if chars.peek() == Some(&'/') {
                    return Ok(());
                } else if chars.peek() == Some(&'!') {
                    chars.next();
                    let _comment = parse_until(&mut chars, '>')?;
                } else {
                    let (tag_name, attrs) = parse_tag(&mut chars)?;
                    let mut element = DomNode {
                        node_type: NodeType::Element(tag_name.clone()),
                        attributes: attrs,
                        children: Vec::new(),
                    };

                    if !is_self_closing(&tag_name) {
                        let remaining: String = chars.by_ref().collect();
                        parse_html_content(&remaining, &mut element)?;
                    }
                    parent.children.push(Box::new(element));
                }
            }
            _ => {
                current_text.push(ch);
                chars.next();
            }
        }
    }

    if !current_text.is_empty() {
        parent.children.push(Box::new(DomNode {
            node_type: NodeType::Text(current_text),
            attributes: HashMap::new(),
            children: Vec::new(),
        }));
    }

    Ok(())
}

fn parse_until(chars: &mut std::iter::Peekable<std::str::Chars>, delimiter: char) -> Result<String, String> {
    let mut result = String::new();
    while let Some(ch) = chars.next() {
        if ch == delimiter {
            return Ok(result);
        }
        result.push(ch);
    }
    Ok(result)
}

fn parse_tag(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<(String, HashMap<String, String>), String> {
    let mut tag_name = String::new();
    let mut attributes = HashMap::new();

    while let Some(&ch) = chars.peek() {
        if ch == ' ' || ch == '>' || ch == '/' {
            break;
        }
        tag_name.push(ch);
        chars.next();
    }

    while chars.peek() == Some(&' ') {
        chars.next();
    }

    while chars.peek() != Some(&'>') {
        if chars.peek() == Some(&'/') {
            chars.next();
            break;
        }

        let mut attr_name = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == '=' || ch == ' ' || ch == '>' {
                break;
            }
            attr_name.push(ch);
            chars.next();
        }

        while chars.peek() == Some(&' ') {
            chars.next();
        }

        let mut attr_value = String::new();
        if chars.peek() == Some(&'=') {
            chars.next();
            while chars.peek() == Some(&' ') {
                chars.next();
            }

            if chars.peek() == Some(&'"') {
                chars.next();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next();
                        break;
                    }
                    attr_value.push(ch);
                    chars.next();
                }
            }
        }

        if !attr_name.is_empty() {
            attributes.insert(attr_name, attr_value);
        }

        while chars.peek() == Some(&' ') {
            chars.next();
        }
    }

    if chars.peek() == Some(&'>') {
        chars.next();
    }

    Ok((tag_name.to_lowercase(), attributes))
}

fn is_self_closing(tag: &str) -> bool {
    matches!(tag, "img" | "br" | "hr" | "input" | "meta" | "link" | "area" | "base" | "col" | "embed" | "param" | "source" | "track" | "wbr")
}
