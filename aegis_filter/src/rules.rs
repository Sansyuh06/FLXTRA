//! Filter rule types and parsing
//!
//! Supports a subset of AdBlock Plus / uBlock Origin filter syntax

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Type of filter rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    /// Block matching resources
    Block,
    /// Allow matching resources (exception)
    Allow,
    /// Cosmetic filter (element hiding)
    Cosmetic,
    /// Comment (ignored)
    Comment,
}

/// Resource type options for matching
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceTypeOptions {
    pub script: Option<bool>,
    pub image: Option<bool>,
    pub stylesheet: Option<bool>,
    pub font: Option<bool>,
    pub media: Option<bool>,
    pub xhr: Option<bool>,
    pub document: Option<bool>,
    pub subdocument: Option<bool>,
    pub websocket: Option<bool>,
    pub other: Option<bool>,
}

/// A parsed filter rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// Original rule text
    pub raw: String,
    /// Rule type
    pub rule_type: RuleType,
    /// Pattern to match
    pub pattern: String,
    /// Is this a regex pattern?
    pub is_regex: bool,
    /// Match at start of URL
    pub match_start: bool,
    /// Match at end of URL
    pub match_end: bool,
    /// Case sensitive matching
    pub case_sensitive: bool,
    /// Resource type restrictions
    pub resource_types: ResourceTypeOptions,
    /// Domains where rule applies
    pub domains: HashSet<String>,
    /// Domains where rule doesn't apply
    pub excluded_domains: HashSet<String>,
    /// Third-party only
    pub third_party: Option<bool>,
    /// First-party only
    pub first_party: Option<bool>,
    /// Cosmetic selector (for element hiding)
    pub selector: Option<String>,
}

impl FilterRule {
    /// Create a simple block rule
    pub fn block(pattern: &str) -> Self {
        Self {
            raw: pattern.to_string(),
            rule_type: RuleType::Block,
            pattern: pattern.to_string(),
            is_regex: false,
            match_start: false,
            match_end: false,
            case_sensitive: false,
            resource_types: ResourceTypeOptions::default(),
            domains: HashSet::new(),
            excluded_domains: HashSet::new(),
            third_party: None,
            first_party: None,
            selector: None,
        }
    }

    /// Create a domain block rule (for DNS-level blocking)
    pub fn block_domain(domain: &str) -> Self {
        Self {
            raw: format!("||{}^", domain),
            rule_type: RuleType::Block,
            pattern: domain.to_string(),
            is_regex: false,
            match_start: true,
            match_end: true,
            case_sensitive: false,
            resource_types: ResourceTypeOptions::default(),
            domains: HashSet::new(),
            excluded_domains: HashSet::new(),
            third_party: None,
            first_party: None,
            selector: None,
        }
    }

    /// Check if this rule matches a URL
    pub fn matches_url(&self, url: &str) -> bool {
        let url_lower = if self.case_sensitive {
            url.to_string()
        } else {
            url.to_lowercase()
        };

        let pattern_lower = if self.case_sensitive {
            self.pattern.clone()
        } else {
            self.pattern.to_lowercase()
        };

        if self.is_regex {
            if let Ok(re) = regex::Regex::new(&self.pattern) {
                return re.is_match(&url_lower);
            }
            return false;
        }

        if self.match_start && self.match_end {
            // Exact match or domain match
            url_lower.contains(&pattern_lower)
        } else if self.match_start {
            // URL starts with pattern (after scheme)
            let without_scheme = url_lower
                .strip_prefix("https://")
                .or_else(|| url_lower.strip_prefix("http://"))
                .unwrap_or(&url_lower);
            without_scheme.starts_with(&pattern_lower)
        } else if self.match_end {
            url_lower.ends_with(&pattern_lower)
        } else {
            url_lower.contains(&pattern_lower)
        }
    }

    /// Check if this rule matches a domain
    pub fn matches_domain(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();
        let pattern_lower = self.pattern.to_lowercase();

        domain_lower == pattern_lower
            || domain_lower.ends_with(&format!(".{}", pattern_lower))
    }

    /// Check if this rule is a domain-level block
    pub fn is_domain_block(&self) -> bool {
        self.rule_type == RuleType::Block && self.match_start && self.match_end
    }
}
