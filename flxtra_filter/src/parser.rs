//! AdBlock Plus / uBlock filter syntax parser

use crate::rules::{FilterRule, RuleType};
use tracing::debug;

/// Parse filter list content into rules
pub fn parse_filter_list(content: &str) -> Vec<FilterRule> {
    content
        .lines()
        .filter_map(|line| parse_filter_rule(line.trim()))
        .collect()
}

/// Parse a single filter rule
pub fn parse_filter_rule(line: &str) -> Option<FilterRule> {
    // Skip empty lines
    if line.is_empty() {
        return None;
    }

    // Comments
    if line.starts_with('!') || line.starts_with('[') {
        return None;
    }

    // Cosmetic filter (element hiding)
    if line.contains("##") || line.contains("#@#") {
        return parse_cosmetic_rule(line);
    }

    // Exception rule
    if line.starts_with("@@") {
        return parse_exception_rule(&line[2..]);
    }

    // Standard block rule
    parse_block_rule(line)
}

/// Parse a block rule
fn parse_block_rule(line: &str) -> Option<FilterRule> {
    let (pattern, options) = split_options(line);
    
    let mut rule = FilterRule::block("");
    rule.raw = line.to_string();
    rule.rule_type = RuleType::Block;

    // Handle domain anchor ||
    let pattern = if pattern.starts_with("||") {
        rule.match_start = true;
        &pattern[2..]
    } else if pattern.starts_with('|') {
        rule.match_start = true;
        &pattern[1..]
    } else {
        pattern
    };

    // Handle end anchor ^
    let pattern = if pattern.ends_with('^') {
        rule.match_end = true;
        &pattern[..pattern.len() - 1]
    } else if pattern.ends_with('|') {
        rule.match_end = true;
        &pattern[..pattern.len() - 1]
    } else {
        pattern
    };

    // Check for regex
    if pattern.starts_with('/') && pattern.ends_with('/') {
        rule.is_regex = true;
        rule.pattern = pattern[1..pattern.len() - 1].to_string();
    } else {
        rule.pattern = pattern.to_string();
    }

    // Parse options
    if let Some(opts) = options {
        parse_options(&mut rule, opts);
    }

    if rule.pattern.is_empty() {
        return None;
    }

    Some(rule)
}

/// Parse an exception rule (@@)
fn parse_exception_rule(line: &str) -> Option<FilterRule> {
    let mut rule = parse_block_rule(line)?;
    rule.rule_type = RuleType::Allow;
    rule.raw = format!("@@{}", line);
    Some(rule)
}

/// Parse a cosmetic filter rule
fn parse_cosmetic_rule(line: &str) -> Option<FilterRule> {
    let is_exception = line.contains("#@#");
    let separator = if is_exception { "#@#" } else { "##" };
    
    let parts: Vec<&str> = line.splitn(2, separator).collect();
    if parts.len() != 2 {
        return None;
    }

    let domains_str = parts[0];
    let selector = parts[1];

    let mut rule = FilterRule::block("");
    rule.raw = line.to_string();
    rule.rule_type = if is_exception { RuleType::Allow } else { RuleType::Cosmetic };
    rule.selector = Some(selector.to_string());

    // Parse domain restrictions
    if !domains_str.is_empty() {
        for domain in domains_str.split(',') {
            let domain = domain.trim();
            if domain.starts_with('~') {
                rule.excluded_domains.insert(domain[1..].to_string());
            } else {
                rule.domains.insert(domain.to_string());
            }
        }
    }

    Some(rule)
}

/// Split rule into pattern and options
fn split_options(line: &str) -> (&str, Option<&str>) {
    // Options are after the last $ not in a regex
    if let Some(pos) = line.rfind('$') {
        // Check if this $ is inside a regex
        let before = &line[..pos];
        if !before.contains('/') || before.matches('/').count() % 2 == 0 {
            return (&line[..pos], Some(&line[pos + 1..]));
        }
    }
    (line, None)
}

/// Parse filter options
fn parse_options(rule: &mut FilterRule, options: &str) {
    for opt in options.split(',') {
        let opt = opt.trim();
        let (negated, opt_name) = if opt.starts_with('~') {
            (true, &opt[1..])
        } else {
            (false, opt)
        };

        match opt_name {
            "script" => rule.resource_types.script = Some(!negated),
            "image" => rule.resource_types.image = Some(!negated),
            "stylesheet" | "css" => rule.resource_types.stylesheet = Some(!negated),
            "font" => rule.resource_types.font = Some(!negated),
            "media" => rule.resource_types.media = Some(!negated),
            "xmlhttprequest" | "xhr" => rule.resource_types.xhr = Some(!negated),
            "document" | "doc" => rule.resource_types.document = Some(!negated),
            "subdocument" => rule.resource_types.subdocument = Some(!negated),
            "websocket" => rule.resource_types.websocket = Some(!negated),
            "other" => rule.resource_types.other = Some(!negated),
            "third-party" | "3p" => rule.third_party = Some(!negated),
            "first-party" | "1p" | "~third-party" => rule.first_party = Some(!negated),
            "match-case" => rule.case_sensitive = true,
            opt if opt.starts_with("domain=") => {
                let domains = &opt[7..];
                for domain in domains.split('|') {
                    if domain.starts_with('~') {
                        rule.excluded_domains.insert(domain[1..].to_string());
                    } else {
                        rule.domains.insert(domain.to_string());
                    }
                }
            }
            _ => {
                // Unknown option, ignore
                debug!("Unknown filter option: {}", opt_name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_block() {
        let rule = parse_filter_rule("||ads.example.com^").unwrap();
        assert_eq!(rule.rule_type, RuleType::Block);
        assert!(rule.match_start);
        assert!(rule.match_end);
        assert_eq!(rule.pattern, "ads.example.com");
    }

    #[test]
    fn test_parse_exception() {
        let rule = parse_filter_rule("@@||example.com^").unwrap();
        assert_eq!(rule.rule_type, RuleType::Allow);
    }

    #[test]
    fn test_parse_cosmetic() {
        let rule = parse_filter_rule("example.com##.ad-banner").unwrap();
        assert_eq!(rule.rule_type, RuleType::Cosmetic);
        assert_eq!(rule.selector, Some(".ad-banner".to_string()));
        assert!(rule.domains.contains("example.com"));
    }
}
