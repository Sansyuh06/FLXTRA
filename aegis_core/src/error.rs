//! Error types for Aegis Browser

use thiserror::Error;

/// Main error type for Aegis Browser
#[derive(Error, Debug)]
pub enum AegisError {
    // Network errors
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("DNS resolution failed: {0}")]
    DnsResolution(String),
    
    #[error("TLS error: {0}")]
    Tls(String),
    
    #[error("HTTP error: {status} - {message}")]
    Http { status: u16, message: String },
    
    #[error("Connection refused: {0}")]
    ConnectionRefused(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),

    // Content errors
    #[error("HTML parse error: {0}")]
    HtmlParse(String),
    
    #[error("CSS parse error: {0}")]
    CssParse(String),
    
    #[error("JavaScript error: {0}")]
    JavaScript(String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    // Security errors
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Blocked by content filter: {0}")]
    ContentBlocked(String),
    
    #[error("Certificate error: {0}")]
    Certificate(String),
    
    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    // System errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IPC error: {0}")]
    Ipc(String),

    // Resource errors
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    // Generic
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Result type alias using AegisError
pub type Result<T> = std::result::Result<T, AegisError>;

impl AegisError {
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            AegisError::Timeout(_)
                | AegisError::ConnectionRefused(_)
                | AegisError::DnsResolution(_)
        )
    }

    /// Check if this is a security-related error
    pub fn is_security_error(&self) -> bool {
        matches!(
            self,
            AegisError::SecurityViolation(_)
                | AegisError::ContentBlocked(_)
                | AegisError::Certificate(_)
                | AegisError::SandboxViolation(_)
        )
    }
}
