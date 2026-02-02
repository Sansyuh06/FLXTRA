//! DNS-over-HTTPS resolver
//!
//! Provides encrypted DNS resolution to prevent DNS-based tracking
//! and man-in-the-middle attacks.

use flxtra_core::{FlxtraError, Result};
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// DNS cache entry
#[derive(Clone)]
struct CacheEntry {
    addresses: Vec<IpAddr>,
    expires_at: Instant,
}

/// DNS-over-HTTPS resolver with caching
pub struct DohResolver {
    resolver: TokioAsyncResolver,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_ttl: Duration,
}

impl DohResolver {
    /// Create a new DoH resolver with the specified server
    pub async fn new(doh_server: &str) -> Result<Self> {
        info!("Initializing DoH resolver with server: {}", doh_server);

        // Configure for DNS-over-HTTPS using Cloudflare
        let config = match doh_server {
            s if s.contains("cloudflare") => ResolverConfig::cloudflare_https(),
            s if s.contains("google") => ResolverConfig::google_https(),
            s if s.contains("quad9") => ResolverConfig::quad9_https(),
            _ => {
                // Default to Cloudflare if unknown
                warn!("Unknown DoH server, defaulting to Cloudflare");
                ResolverConfig::cloudflare_https()
            }
        };

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(5);
        opts.attempts = 2;
        opts.validate = true; // Enable DNSSEC validation

        let resolver = TokioAsyncResolver::tokio(config, opts);

        Ok(Self {
            resolver,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minute cache
        })
    }

    /// Resolve a hostname to IP addresses
    pub async fn resolve(&self, hostname: &str) -> Result<Vec<IpAddr>> {
        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(entry) = cache.get(hostname) {
                if entry.expires_at > Instant::now() {
                    debug!("DNS cache hit for {}", hostname);
                    return Ok(entry.addresses.clone());
                }
            }
        }

        debug!("Resolving {} via DoH", hostname);

        // Perform DoH lookup
        let response = self
            .resolver
            .lookup_ip(hostname)
            .await
            .map_err(|e| FlxtraError::DnsResolution(format!("{}: {}", hostname, e)))?;

        let addresses: Vec<IpAddr> = response.iter().collect();

        if addresses.is_empty() {
            return Err(FlxtraError::DnsResolution(format!(
                "No addresses found for {}",
                hostname
            )));
        }

        // Cache the result
        {
            let mut cache = self.cache.write();
            cache.insert(
                hostname.to_string(),
                CacheEntry {
                    addresses: addresses.clone(),
                    expires_at: Instant::now() + self.cache_ttl,
                },
            );
        }

        debug!("Resolved {} to {:?}", hostname, addresses);
        Ok(addresses)
    }

    /// Clear the DNS cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        info!("DNS cache cleared");
    }

    /// Flush expired entries from cache
    pub fn flush_expired(&self) {
        let mut cache = self.cache.write();
        let now = Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_doh_resolver() {
        let resolver = DohResolver::new("https://cloudflare-dns.com/dns-query")
            .await
            .unwrap();

        let addrs = resolver.resolve("example.com").await.unwrap();
        assert!(!addrs.is_empty());
    }
}
