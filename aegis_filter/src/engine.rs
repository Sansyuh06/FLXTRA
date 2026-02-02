//! Filter engine implementation
//!
//! High-performance filtering using bloom filters and pattern matching

use aegis_core::{Origin, ResourceType};
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, info};

use crate::parser::parse_filter_list;
use crate::rules::{FilterRule, RuleType};

/// Built-in ad/tracker domains (subset for quick blocking)
const BUILTIN_AD_DOMAINS: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "googleadservices.com",
    "google-analytics.com",
    "googletagmanager.com",
    "facebook.com/tr",
    "connect.facebook.net",
    "ad.doubleclick.net",
    "pagead2.googlesyndication.com",
    "adservice.google.com",
    "ads.twitter.com",
    "analytics.twitter.com",
    "ads.linkedin.com",
    "bat.bing.com",
    "advertising.com",
    "adnxs.com",
    "criteo.com",
    "criteo.net",
    "outbrain.com",
    "taboola.com",
    "amazon-adsystem.com",
    "moatads.com",
    "quantserve.com",
    "scorecardresearch.com",
    "hotjar.com",
    "fullstory.com",
    "mixpanel.com",
    "segment.io",
    "segment.com",
    "amplitude.com",
    "branch.io",
    "appsflyer.com",
    "adjust.com",
    "chartbeat.com",
    "newrelic.com",
    "nr-data.net",
    "omtrdc.net",
    "demdex.net",
    "krxd.net",
    "bluekai.com",
    "exelator.com",
    "rlcdn.com",
    "tapad.com",
    "casalemedia.com",
    "pubmatic.com",
    "rubiconproject.com",
    "openx.net",
    "contextweb.com",
    "spotxchange.com",
    "indexexchange.com",
];

/// High-performance filter engine
pub struct FilterEngine {
    /// Block rules
    block_rules: RwLock<Vec<FilterRule>>,
    /// Exception rules
    allow_rules: RwLock<Vec<FilterRule>>,
    /// Cosmetic rules
    cosmetic_rules: RwLock<Vec<FilterRule>>,
    /// Blocked domains set (fast lookup)
    blocked_domains: RwLock<HashSet<String>>,
    /// Statistics
    blocked_count: AtomicU64,
}

impl FilterEngine {
    /// Create a new filter engine with built-in rules
    pub fn new() -> Self {
        let mut blocked_domains = HashSet::new();
        
        // Add built-in ad domains
        for domain in BUILTIN_AD_DOMAINS {
            blocked_domains.insert(domain.to_string());
        }

        info!("Filter engine initialized with {} built-in domains", blocked_domains.len());

        Self {
            block_rules: RwLock::new(Vec::new()),
            allow_rules: RwLock::new(Vec::new()),
            cosmetic_rules: RwLock::new(Vec::new()),
            blocked_domains: RwLock::new(blocked_domains),
            blocked_count: AtomicU64::new(0),
        }
    }

    /// Add rules from filter list content
    pub fn add_rules(&self, content: &str) -> usize {
        let rules = parse_filter_list(content);
        let count = rules.len();

        let mut block_rules = self.block_rules.write();
        let mut allow_rules = self.allow_rules.write();
        let mut cosmetic_rules = self.cosmetic_rules.write();
        let mut blocked_domains = self.blocked_domains.write();

        for rule in rules {
            // Extract blocked domains for fast lookup
            if rule.is_domain_block() {
                blocked_domains.insert(rule.pattern.clone());
            }

            match rule.rule_type {
                RuleType::Block => block_rules.push(rule),
                RuleType::Allow => allow_rules.push(rule),
                RuleType::Cosmetic => cosmetic_rules.push(rule),
                RuleType::Comment => {}
            }
        }

        info!("Added {} filter rules", count);
        count
    }

    /// Add a single rule
    pub fn add_rule(&self, rule_text: &str) {
        if let Some(rule) = crate::parser::parse_filter_rule(rule_text) {
            match rule.rule_type {
                RuleType::Block => {
                    if rule.is_domain_block() {
                        self.blocked_domains.write().insert(rule.pattern.clone());
                    }
                    self.block_rules.write().push(rule);
                }
                RuleType::Allow => self.allow_rules.write().push(rule),
                RuleType::Cosmetic => self.cosmetic_rules.write().push(rule),
                RuleType::Comment => {}
            }
        }
    }

    /// Check if a URL should be blocked
    pub fn should_block(
        &self,
        url: &str,
        resource_type: ResourceType,
        origin: Option<&Origin>,
    ) -> bool {
        // Quick domain check first
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                if self.should_block_host(host) {
                    self.blocked_count.fetch_add(1, Ordering::Relaxed);
                    return true;
                }
            }
        }

        // Check exception rules first
        let allow_rules = self.allow_rules.read();
        for rule in allow_rules.iter() {
            if self.rule_matches(rule, url, resource_type, origin) {
                debug!("Allowed by exception: {}", url);
                return false;
            }
        }

        // Check block rules
        let block_rules = self.block_rules.read();
        for rule in block_rules.iter() {
            if self.rule_matches(rule, url, resource_type, origin) {
                self.blocked_count.fetch_add(1, Ordering::Relaxed);
                debug!("Blocked: {}", url);
                return true;
            }
        }

        false
    }

    /// Check if a host should be blocked (DNS-level)
    pub fn should_block_host(&self, host: &str) -> bool {
        let blocked = self.blocked_domains.read();
        let host_lower = host.to_lowercase();

        // Direct match
        if blocked.contains(&host_lower) {
            return true;
        }

        // Check parent domains
        let parts: Vec<&str> = host_lower.split('.').collect();
        for i in 1..parts.len().saturating_sub(1) {
            let parent = parts[i..].join(".");
            if blocked.contains(&parent) {
                return true;
            }
        }

        false
    }

    /// Check if a single rule matches
    fn rule_matches(
        &self,
        rule: &FilterRule,
        url: &str,
        resource_type: ResourceType,
        origin: Option<&Origin>,
    ) -> bool {
        // Check resource type
        if !self.resource_type_matches(rule, resource_type) {
            return false;
        }

        // Check third-party
        if let Some(third_party) = rule.third_party {
            let is_third_party = self.is_third_party(url, origin);
            if third_party != is_third_party {
                return false;
            }
        }

        // Check first-party
        if let Some(first_party) = rule.first_party {
            let is_first_party = !self.is_third_party(url, origin);
            if first_party != is_first_party {
                return false;
            }
        }

        // Check domain restrictions
        if !rule.domains.is_empty() {
            if let Some(o) = origin {
                if !rule.domains.contains(&o.host) {
                    return false;
                }
            }
        }

        // Check URL pattern
        rule.matches_url(url)
    }

    /// Check if resource type matches rule
    fn resource_type_matches(&self, rule: &FilterRule, resource_type: ResourceType) -> bool {
        let opts = &rule.resource_types;

        // If no specific resource types are set, match all
        if opts.script.is_none()
            && opts.image.is_none()
            && opts.stylesheet.is_none()
            && opts.font.is_none()
            && opts.media.is_none()
            && opts.xhr.is_none()
            && opts.document.is_none()
            && opts.subdocument.is_none()
            && opts.websocket.is_none()
            && opts.other.is_none()
        {
            return true;
        }

        match resource_type {
            ResourceType::Script => opts.script.unwrap_or(false),
            ResourceType::Image => opts.image.unwrap_or(false),
            ResourceType::Stylesheet => opts.stylesheet.unwrap_or(false),
            ResourceType::Font => opts.font.unwrap_or(false),
            ResourceType::Media => opts.media.unwrap_or(false),
            ResourceType::Xhr | ResourceType::Fetch => opts.xhr.unwrap_or(false),
            ResourceType::Document => opts.document.unwrap_or(false),
            ResourceType::WebSocket => opts.websocket.unwrap_or(false),
            ResourceType::Other => opts.other.unwrap_or(false),
        }
    }

    /// Check if request is third-party
    fn is_third_party(&self, url: &str, origin: Option<&Origin>) -> bool {
        let origin = match origin {
            Some(o) => o,
            None => return false,
        };

        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return !host.ends_with(&origin.host) && origin.host != host;
            }
        }

        false
    }

    /// Get cosmetic rules for a page
    pub fn get_cosmetic_rules(&self, domain: &str) -> Vec<String> {
        let rules = self.cosmetic_rules.read();
        let mut selectors = Vec::new();

        for rule in rules.iter() {
            if let Some(selector) = &rule.selector {
                // Check domain restrictions
                if rule.domains.is_empty() || rule.domains.contains(domain) {
                    if !rule.excluded_domains.contains(domain) {
                        selectors.push(selector.clone());
                    }
                }
            }
        }

        selectors
    }

    /// Get blocked count
    pub fn blocked_count(&self) -> u64 {
        self.blocked_count.load(Ordering::Relaxed)
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.blocked_count.store(0, Ordering::Relaxed);
    }

    /// Get rule counts
    pub fn rule_counts(&self) -> (usize, usize, usize) {
        (
            self.block_rules.read().len(),
            self.allow_rules.read().len(),
            self.cosmetic_rules.read().len(),
        )
    }
}

impl Default for FilterEngine {
    fn default() -> Self {
        Self::new()
    }
}
