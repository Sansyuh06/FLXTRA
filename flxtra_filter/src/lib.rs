//! Flxtra Filter Engine
//!
//! High-performance ad and tracker blocking with:
//! - Network-level URL filtering
//! - DNS-level domain blocking
//! - EasyList/uBlock filter syntax support
//! - Cosmetic filtering rules

pub mod engine;
pub mod parser;
pub mod rules;

pub use engine::FilterEngine;
pub use rules::{FilterRule, RuleType};
