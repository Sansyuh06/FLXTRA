//! MCP Client
use crate::capabilities::{Capability, CapabilityGrant, default_grants};
use crate::protocol::{McpMessage, McpRequest, McpResponse};
use std::collections::HashMap;
use tracing::{info, warn};

pub struct McpClient {
    grants: HashMap<Capability, CapabilityGrant>,
    pending_requests: HashMap<String, McpRequest>,
}

impl McpClient {
    pub fn new() -> Self {
        let grants = default_grants().into_iter().map(|g| (g.capability, g)).collect();
        Self { grants, pending_requests: HashMap::new() }
    }

    pub fn can_perform(&self, cap: Capability) -> bool {
        self.grants.get(&cap).map(|g| g.allowed).unwrap_or(false)
    }

    pub fn requires_confirmation(&self, cap: Capability) -> bool {
        self.grants.get(&cap).map(|g| g.requires_user_confirm).unwrap_or(true)
    }

    pub fn handle_request(&mut self, request: McpRequest) -> McpResponse {
        info!("MCP request: {}", request.method);
        
        let cap = match request.method.as_str() {
            "navigate" => Capability::Navigate,
            "read_page" | "get_content" => Capability::ReadPage,
            "click" => Capability::Click,
            "type" | "input" => Capability::Type,
            "screenshot" => Capability::Screenshot,
            "execute_script" => Capability::ExecuteScript,
            _ => {
                warn!("Unknown MCP method: {}", request.method);
                return McpResponse::error(request.id, -32601, "Method not found");
            }
        };

        if !self.can_perform(cap) {
            return McpResponse::error(request.id, -32000, "Capability not granted");
        }

        if self.requires_confirmation(cap) {
            self.pending_requests.insert(request.id.clone(), request.clone());
            return McpResponse::error(request.id, -32001, "Awaiting user confirmation");
        }

        self.execute_request(&request)
    }

    pub fn confirm_request(&mut self, request_id: &str) -> Option<McpResponse> {
        self.pending_requests.remove(request_id).map(|req| self.execute_request(&req))
    }

    pub fn deny_request(&mut self, request_id: &str) -> Option<McpResponse> {
        self.pending_requests.remove(request_id).map(|req| 
            McpResponse::error(req.id, -32002, "User denied request")
        )
    }

    fn execute_request(&self, request: &McpRequest) -> McpResponse {
        // Actual execution would call browser APIs
        McpResponse::success(request.id.clone(), serde_json::json!({"status": "ok"}))
    }
}

impl Default for McpClient {
    fn default() -> Self { Self::new() }
}
