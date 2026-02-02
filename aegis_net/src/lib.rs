//! Aegis Network Stack
//!
//! Privacy-first network implementation with:
//! - DNS-over-HTTPS (DoH) resolution
//! - HTTPS-only connections
//! - Integrated ad/tracker filtering
//! - First-party isolation

pub mod dns;
pub mod http;
pub mod client;

pub use client::NetworkClient;
pub use dns::DohResolver;
pub use http::HttpClient;
