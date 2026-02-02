//! CSS parser

use crate::properties::{parse_property, Property};
use crate::stylesheet::{MediaRule, Selector, StyleDeclaration, StyleRule, StyleSheet};
use tracing::debug;

/// CSS parser
pub struct CssParser;

impl CssParser {
    /// Parse CSS content into a stylesheet
    pub fn parse(css: &str) -> StyleSheet {
        let mut stylesheet = StyleSheet::new();
        let mut current_pos = 0;
        let chars: Vec<char> = css.chars().collect();
        let len = chars.len();

        while current_pos < len {
            // Skip whitespace
            while current_pos < len && chars[current_pos].is_whitespace() {
                current_pos += 1;
            }

            if current_pos >= len {
                break;
            }

            // Check for at-rules
            if chars[current_pos] == '@' {
                if let Some((rule, end_pos)) = Self::parse_at_rule(&chars, current_pos) {
                    stylesheet.add_rule(rule);
                    current_pos = end_pos;
                    continue;
                }
            }

            // Check for comment
            if current_pos + 1 < len && chars[current_pos] == '/' && chars[current_pos + 1] == '*' {
                // Skip comment
                current_pos += 2;
                while current_pos + 1 < len {
                    if chars[current_pos] == '*' && chars[current_pos + 1] == '/' {
                        current_pos += 2;
                        break;
                    }
                    current_pos += 1;
                }
                continue;
            }

            // Parse style rule
            if let Some((rule, end_pos)) = Self::parse_style_rule(&chars, current_pos) {
                stylesheet.add_rule(rule);
                current_pos = end_pos;
            } else {
                current_pos += 1;
            }
        }

        debug!("Parsed {} CSS rules", stylesheet.rules.len());
        stylesheet
    }

    fn parse_at_rule(chars: &[char], start: usize) -> Option<(StyleRule, usize)> {
        let len = chars.len();
        let mut pos = start + 1; // Skip @

        // Get at-rule name
        let mut name = String::new();
        while pos < len && chars[pos].is_alphanumeric() {
            name.push(chars[pos]);
            pos += 1;
        }

        match name.as_str() {
            "import" => {
                // Find the URL
                while pos < len && chars[pos] != ';' {
                    pos += 1;
                }
                let url_part: String = chars[start..pos].iter().collect();
                pos += 1; // Skip ;
                
                // Extract URL from @import url(...) or @import "..."
                let url = url_part
                    .replace("@import", "")
                    .replace("url(", "")
                    .replace(')', "")
                    .replace('"', "")
                    .replace('\'', "")
                    .trim()
                    .to_string();
                
                Some((StyleRule::Import(url), pos))
            }
            "media" => {
                // Get media query
                let mut query = String::new();
                while pos < len && chars[pos] != '{' {
                    query.push(chars[pos]);
                    pos += 1;
                }
                pos += 1; // Skip {

                // Parse rules inside media query
                let mut rules = Vec::new();
                let mut brace_count = 1;
                let inner_start = pos;

                while pos < len && brace_count > 0 {
                    if chars[pos] == '{' {
                        brace_count += 1;
                    } else if chars[pos] == '}' {
                        brace_count -= 1;
                    }
                    pos += 1;
                }

                // Parse inner content
                let inner: String = chars[inner_start..pos - 1].iter().collect();
                let inner_sheet = Self::parse(&inner);
                for rule in inner_sheet.rules {
                    if let StyleRule::Style(decl) = rule {
                        rules.push(decl);
                    }
                }

                Some((
                    StyleRule::Media(MediaRule {
                        query: query.trim().to_string(),
                        rules,
                    }),
                    pos,
                ))
            }
            "font-face" => {
                // Skip to opening brace
                while pos < len && chars[pos] != '{' {
                    pos += 1;
                }
                pos += 1; // Skip {

                // Parse properties
                let mut props = Vec::new();
                while pos < len && chars[pos] != '}' {
                    if let Some((prop, end)) = Self::parse_property(chars, pos) {
                        props.push(prop);
                        pos = end;
                    } else {
                        pos += 1;
                    }
                }
                pos += 1; // Skip }

                Some((StyleRule::FontFace(props), pos))
            }
            _ => {
                // Skip unknown at-rule
                let mut brace_count = 0;
                while pos < len {
                    if chars[pos] == '{' {
                        brace_count += 1;
                    } else if chars[pos] == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            pos += 1;
                            break;
                        }
                    } else if chars[pos] == ';' && brace_count == 0 {
                        pos += 1;
                        break;
                    }
                    pos += 1;
                }
                None
            }
        }
    }

    fn parse_style_rule(chars: &[char], start: usize) -> Option<(StyleRule, usize)> {
        let len = chars.len();
        let mut pos = start;

        // Get selectors (up to {)
        let mut selector_str = String::new();
        while pos < len && chars[pos] != '{' {
            selector_str.push(chars[pos]);
            pos += 1;
        }

        if pos >= len {
            return None;
        }

        pos += 1; // Skip {

        // Parse selectors
        let selectors: Vec<Selector> = selector_str
            .split(',')
            .map(|s| Selector::parse(s.trim()))
            .collect();

        // Parse properties
        let mut properties = Vec::new();
        while pos < len && chars[pos] != '}' {
            if let Some((prop, end)) = Self::parse_property(chars, pos) {
                properties.push(prop);
                pos = end;
            } else {
                pos += 1;
            }
        }

        pos += 1; // Skip }

        Some((
            StyleRule::Style(StyleDeclaration {
                selectors,
                properties,
            }),
            pos,
        ))
    }

    fn parse_property(chars: &[char], start: usize) -> Option<(Property, usize)> {
        let len = chars.len();
        let mut pos = start;

        // Skip whitespace
        while pos < len && (chars[pos].is_whitespace() || chars[pos] == ';') {
            pos += 1;
        }

        if pos >= len || chars[pos] == '}' {
            return None;
        }

        // Get property name
        let mut name = String::new();
        while pos < len && chars[pos] != ':' && chars[pos] != '}' {
            name.push(chars[pos]);
            pos += 1;
        }

        if pos >= len || chars[pos] != ':' {
            return None;
        }
        pos += 1; // Skip :

        // Get property value
        let mut value = String::new();
        while pos < len && chars[pos] != ';' && chars[pos] != '}' {
            value.push(chars[pos]);
            pos += 1;
        }

        if pos < len && chars[pos] == ';' {
            pos += 1;
        }

        let name = name.trim();
        let value = value.trim();

        parse_property(name, value).map(|prop| (prop, pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let css = "body { color: red; background-color: #fff; }";
        let sheet = CssParser::parse(css);
        assert_eq!(sheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_multiple_rules() {
        let css = r#"
            body { color: black; }
            .container { width: 100%; }
            #main { padding: 20px; }
        "#;
        let sheet = CssParser::parse(css);
        assert_eq!(sheet.rules.len(), 3);
    }
}
