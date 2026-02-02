//! Core types used throughout Aegis Browser

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

/// Unique identifier for a browser tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(Uuid);

impl TabId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TabId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a browsing context (iframe, window)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextId(Uuid);

impl ContextId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ContextId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a container (isolation domain)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerId(Uuid);

impl ContainerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Default container for general browsing
    pub fn default_container() -> Self {
        Self(Uuid::from_u128(0))
    }
}

impl Default for ContainerId {
    fn default() -> Self {
        Self::default_container()
    }
}

/// Origin representation for security decisions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Origin {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
}

impl Origin {
    pub fn from_url(url: &Url) -> Option<Self> {
        Some(Self {
            scheme: url.scheme().to_string(),
            host: url.host_str()?.to_string(),
            port: url.port(),
        })
    }

    pub fn is_same_origin(&self, other: &Origin) -> bool {
        self.scheme == other.scheme && self.host == other.host && self.port == other.port
    }

    /// Get the effective TLD+1 for cookie partitioning
    pub fn etld_plus_one(&self) -> String {
        // Simplified - in production would use public suffix list
        let parts: Vec<&str> = self.host.split('.').collect();
        if parts.len() >= 2 {
            parts[parts.len() - 2..].join(".")
        } else {
            self.host.clone()
        }
    }
}

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Patch => "PATCH",
        }
    }
}

/// Resource type for content filtering decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Document,
    Script,
    Stylesheet,
    Image,
    Font,
    Media,
    Xhr,
    Fetch,
    WebSocket,
    Other,
}

/// Navigation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationRequest {
    pub url: String,
    pub tab_id: TabId,
    pub container_id: ContainerId,
    pub referrer: Option<String>,
    pub is_user_initiated: bool,
}

/// Page load state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadState {
    NotStarted,
    Loading,
    Interactive,
    Complete,
    Failed,
}

/// Security state for a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityState {
    pub is_secure: bool,
    pub certificate_valid: bool,
    pub mixed_content: bool,
    pub blocked_resources: u32,
    pub container_id: ContainerId,
}

impl Default for SecurityState {
    fn default() -> Self {
        Self {
            is_secure: false,
            certificate_valid: false,
            mixed_content: false,
            blocked_resources: 0,
            container_id: ContainerId::default(),
        }
    }
}

/// Shared reference type
pub type Shared<T> = Arc<T>;
