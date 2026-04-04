//! CSS Parser & Style Resolution
//! 
//! Responsible for:
//! - Parsing CSS stylesheets
//! - Computing resolved styles by matching selectors against DOM tree
//! - Resolving cascading, inheritance, and specificity rules
//! - Outputting a styled DOM tree for layout engine

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct CSSRule {
    pub selector: String,
    pub properties: HashMap<String, String>,
    pub specificity: (u32, u32, u32), // (id, class, element)
}

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub properties: HashMap<String, String>,
}

pub struct CssParser;

impl CssParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(css: &str) -> Result<Vec<CSSRule>, String> {
        let mut rules = Vec::new();
        let mut current_selector = String::new();
        let mut current_properties = String::new();
        let mut in_rule = false;
        let mut brace_count = 0;

        for ch in css.chars() {
            match ch {
                '{' => {
                    if !in_rule {
                        current_selector = current_selector.trim().to_string();
                        in_rule = true;
                    }
                    brace_count += 1;
                }
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 && in_rule {
                        let props = parse_properties(&current_properties);
                        let specificity = calculate_specificity(&current_selector);
                        rules.push(CSSRule {
                            selector: current_selector.clone(),
                            properties: props,
                            specificity,
                        });
                        current_selector.clear();
                        current_properties.clear();
                        in_rule = false;
                    }
                }
                _ => {
                    if in_rule {
                        current_properties.push(ch);
                    } else {
                        current_selector.push(ch);
                    }
                }
            }
        }

        Ok(rules)
    }
}

fn parse_properties(property_str: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();
    for declaration in property_str.split(';') {
        if let Some((key, value)) = declaration.split_once(':') {
            props.insert(
                key.trim().to_lowercase(),
                value.trim().to_string(),
            );
        }
    }
    props
}

fn calculate_specificity(selector: &str) -> (u32, u32, u32) {
    let mut ids = 0u32;
    let mut classes = 0u32;
    let mut elements = 0u32;

    for part in selector.split_whitespace() {
        if part.starts_with('#') {
            ids += 1;
        } else if part.starts_with('.') {
            classes += 1;
        } else if !part.is_empty() && !part.starts_with('*') {
            elements += 1;
        }
    }

    (ids, classes, elements)
}
