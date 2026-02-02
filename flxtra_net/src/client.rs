//! Unified network client
//!
//! Combines DNS resolver, HTTP client, and content filter
//! into a single privacy-focused network interface.

use flxtra_core::{FlxtraError, HttpMethod, Origin, ResourceType, Result};
use flxtra_filter::FilterEngine;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::dns::DohResolver;
use crate::http::{HttpClient, HttpResponse};

/// Main network client for the browser
pub struct NetworkClient {
    dns: Arc<DohResolver>,
    http: Arc<HttpClient>,
    filter: Arc<FilterEngine>,
}

impl NetworkClient {
    /// Create a new network client
    pub async fn new(
        doh_server: &str,
        allow_http: bool,
        timeout_secs: u64,
    ) -> Result<Self> {
        let dns = Arc::new(DohResolver::new(doh_server).await?);
        let http = Arc::new(HttpClient::new(allow_http, timeout_secs)?);
        let filter = Arc::new(FilterEngine::new());

        info!("Network client initialized");

        Ok(Self { dns, http, filter })
    }

    /// Load filter lists
    pub async fn load_filters(&self, lists: &[String]) -> Result<()> {
        for list_url in lists {
            match self.http.get(list_url).await {
                Ok(response) => {
                    if let Ok(content) = String::from_utf8(response.body.to_vec()) {
                        let count = self.filter.add_rules(&content);
                        info!("Loaded {} rules from {}", count, list_url);
                    }
                }
                Err(e) => {
                    warn!("Failed to load filter list {}: {}", list_url, e);
                }
            }
        }
        Ok(())
    }

    /// Fetch a resource with filtering
    pub async fn fetch(
        &self,
        url: &str,
        resource_type: ResourceType,
        origin: Option<&Origin>,
    ) -> Result<HttpResponse> {
        // Check content filter first
        if self.filter.should_block(url, resource_type, origin) {
            debug!("Blocked by filter: {}", url);
            return Err(FlxtraError::ContentBlocked(url.to_string()));
        }

        // Perform the request
        self.http.get(url).await
    }

    /// Resolve a hostname
    pub async fn resolve_dns(&self, hostname: &str) -> Result<Vec<std::net::IpAddr>> {
        // Check if hostname should be blocked (DNS-level ad blocking)
        if self.filter.should_block_host(hostname) {
            debug!("Blocked host: {}", hostname);
            return Err(FlxtraError::ContentBlocked(hostname.to_string()));
        }

        self.dns.resolve(hostname).await
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        self.dns.clear_cache();
        info!("All network caches cleared");
    }

    /// Get blocked resource count
    pub fn blocked_count(&self) -> u64 {
        self.filter.blocked_count()
    }

    /// Add custom filter rule
    pub fn add_custom_rule(&self, rule: &str) {
        self.filter.add_rule(rule);
    }

    /// Check if a URL would be blocked
    pub fn would_block(&self, url: &str, resource_type: ResourceType) -> bool {
        self.filter.should_block(url, resource_type, None)
    }
}
