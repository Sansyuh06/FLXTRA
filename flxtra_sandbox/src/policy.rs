//! Sandbox policy configuration
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub allow_clipboard: bool,
    pub allow_camera: bool,
    pub allow_microphone: bool,
    pub allow_geolocation: bool,
    pub blocked_hosts: Vec<String>,
    pub max_memory_mb: u32,
    pub max_cpu_percent: u32,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            allow_network: true,
            allow_filesystem: false,
            allow_clipboard: false,
            allow_camera: false,
            allow_microphone: false,
            allow_geolocation: false,
            blocked_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string(), "0.0.0.0".to_string()],
            max_memory_mb: 512,
            max_cpu_percent: 50,
        }
    }
}

impl SandboxPolicy {
    pub fn strict() -> Self {
        Self { 
            allow_network: false, 
            allow_clipboard: false,
            max_memory_mb: 256, 
            max_cpu_percent: 25, 
            ..Default::default() 
        }
    }
    
    pub fn permissive() -> Self {
        Self {
            allow_network: true,
            allow_filesystem: true,
            allow_clipboard: true,
            max_memory_mb: 1024,
            max_cpu_percent: 100,
            ..Default::default()
        }
    }
    
    pub fn is_host_allowed(&self, host: &str) -> bool {
        !self.blocked_hosts.iter().any(|h| host.contains(h))
    }
}
