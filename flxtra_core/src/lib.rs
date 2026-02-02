//! Flxtra Core - Shared types and utilities for Flxtra Browser
//!
//! This crate provides fundamental types, error handling, and utilities
//! used across all browser components.

pub mod error;
pub mod types;
pub mod config;
pub mod ipc;

pub use error::{FlxtraError, Result};
pub use types::*;
pub use config::BrowserConfig;
