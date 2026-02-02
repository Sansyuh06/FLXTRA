//! Browser configuration system

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Privacy level presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyLevel {
    /// Standard privacy protections
    Standard,
    /// Aggressive privacy protections (may break some sites)
    Strict,
    /// Maximum privacy (Tor-like, will break many sites)
    Maximum,
}

impl Default for PrivacyLevel {
    fn default() -> Self {
        Self::Strict
    }
}

/// Performance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceMode {
    /// Text only, no JS, no images
    TextOnly,
    /// Basic rendering, interpreter JS only
    Lite,
    /// Full rendering with baseline JIT
    Standard,
}

impl Default for PerformanceMode {
    fn default() -> Self {
        Self::Lite
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// DNS-over-HTTPS server URL
    pub doh_server: String,
    /// Allow HTTP (insecure) connections
    pub allow_http: bool,
    /// Connection timeout in seconds
    pub timeout_seconds: u32,
    /// Maximum concurrent connections per host
    pub max_connections_per_host: u32,
    /// Enable QUIC/HTTP3
    pub enable_http3: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            doh_server: "https://cloudflare-dns.com/dns-query".to_string(),
            allow_http: false,
            timeout_seconds: 30,
            max_connections_per_host: 6,
            enable_http3: true,
        }
    }
}

/// Content filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Enable ad blocking
    pub block_ads: bool,
    /// Enable tracker blocking
    pub block_trackers: bool,
    /// Enable fingerprint protection
    pub fingerprint_protection: bool,
    /// Block third-party cookies
    pub block_third_party_cookies: bool,
    /// Block all cookies
    pub block_all_cookies: bool,
    /// Custom block rules (adblock syntax)
    pub custom_rules: Vec<String>,
    /// Domains to whitelist
    pub whitelist: Vec<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            block_ads: true,
            block_trackers: true,
            fingerprint_protection: true,
            block_third_party_cookies: true,
            block_all_cookies: false,
            custom_rules: Vec::new(),
            whitelist: Vec::new(),
        }
    }
}

/// JavaScript configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaScriptConfig {
    /// Enable JavaScript execution
    pub enabled: bool,
    /// Enable JIT compilation (less secure)
    pub enable_jit: bool,
    /// Script execution timeout in milliseconds
    pub timeout_ms: u32,
    /// Maximum heap size in MB
    pub max_heap_mb: u32,
}

impl Default for JavaScriptConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enable_jit: false, // Disabled by default for security
            timeout_ms: 30000,
            max_heap_mb: 512,
        }
    }
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable process sandboxing
    pub enabled: bool,
    /// Enable per-site process isolation
    pub site_isolation: bool,
    /// Enable container-based isolation
    pub container_isolation: bool,
    /// Block local network access
    pub block_local_network: bool,
    /// Block file system access
    pub block_filesystem: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            site_isolation: true,
            container_isolation: true,
            block_local_network: true,
            block_filesystem: true,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Window width
    pub window_width: u32,
    /// Window height  
    pub window_height: u32,
    /// Enable dark mode
    pub dark_mode: bool,
    /// Show blocked resource count
    pub show_blocked_count: bool,
    /// Homepage URL
    pub homepage: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            dark_mode: true,
            show_blocked_count: true,
            homepage: "aegis://newtab".to_string(),
        }
    }
}

/// Main browser configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Privacy level preset
    pub privacy_level: PrivacyLevel,
    /// Performance mode
    pub performance_mode: PerformanceMode,
    /// Network settings
    pub network: NetworkConfig,
    /// Content filtering
    pub filter: FilterConfig,
    /// JavaScript settings
    pub javascript: JavaScriptConfig,
    /// Sandbox settings
    pub sandbox: SandboxConfig,
    /// UI settings
    pub ui: UiConfig,
    /// Profile directory
    pub profile_dir: PathBuf,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            privacy_level: PrivacyLevel::default(),
            performance_mode: PerformanceMode::default(),
            network: NetworkConfig::default(),
            filter: FilterConfig::default(),
            javascript: JavaScriptConfig::default(),
            sandbox: SandboxConfig::default(),
            ui: UiConfig::default(),
            profile_dir: PathBuf::from("./profile"),
        }
    }
}

impl BrowserConfig {
    /// Load configuration from file
    pub fn load(path: &PathBuf) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| crate::AegisError::Config(e.to_string()))
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> crate::Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::AegisError::Config(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Apply privacy preset
    pub fn apply_privacy_preset(&mut self, level: PrivacyLevel) {
        self.privacy_level = level;
        match level {
            PrivacyLevel::Standard => {
                self.filter.block_ads = true;
                self.filter.block_trackers = true;
                self.filter.fingerprint_protection = false;
                self.javascript.enabled = true;
            }
            PrivacyLevel::Strict => {
                self.filter.block_ads = true;
                self.filter.block_trackers = true;
                self.filter.fingerprint_protection = true;
                self.filter.block_third_party_cookies = true;
                self.javascript.enabled = true;
                self.javascript.enable_jit = false;
            }
            PrivacyLevel::Maximum => {
                self.filter.block_ads = true;
                self.filter.block_trackers = true;
                self.filter.fingerprint_protection = true;
                self.filter.block_all_cookies = true;
                self.javascript.enabled = false;
                self.sandbox.block_local_network = true;
            }
        }
    }
}
