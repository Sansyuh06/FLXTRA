//! MCP Protocol types
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpMessage {
    Request(McpRequest),
    Response(McpResponse),
    Notification { method: String, params: serde_json::Value },
}

impl McpRequest {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), method: method.to_string(), params }
    }
}

impl McpResponse {
    pub fn success(id: String, result: serde_json::Value) -> Self {
        Self { id, result: Some(result), error: None }
    }
    pub fn error(id: String, code: i32, message: &str) -> Self {
        Self { id, result: None, error: Some(McpError { code, message: message.to_string() }) }
    }
}
