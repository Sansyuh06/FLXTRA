//! Flxtra MCP - Model Context Protocol client for AI assistants
pub mod client;
pub mod protocol;
pub mod capabilities;
pub use client::McpClient;
pub use capabilities::Capability;
