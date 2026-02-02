//! MCP Capabilities
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    Navigate,
    ReadPage,
    Click,
    Type,
    Screenshot,
    ExecuteScript,
    ManageTabs,
    AccessHistory,
    AccessBookmarks,
    ModifySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub capability: Capability,
    pub allowed: bool,
    pub requires_user_confirm: bool,
}

impl Default for CapabilityGrant {
    fn default() -> Self {
        Self { capability: Capability::ReadPage, allowed: false, requires_user_confirm: true }
    }
}

pub fn default_grants() -> Vec<CapabilityGrant> {
    vec![
        CapabilityGrant { capability: Capability::Navigate, allowed: true, requires_user_confirm: true },
        CapabilityGrant { capability: Capability::ReadPage, allowed: true, requires_user_confirm: false },
        CapabilityGrant { capability: Capability::Click, allowed: true, requires_user_confirm: true },
        CapabilityGrant { capability: Capability::Type, allowed: true, requires_user_confirm: true },
        CapabilityGrant { capability: Capability::Screenshot, allowed: true, requires_user_confirm: true },
        CapabilityGrant { capability: Capability::ExecuteScript, allowed: false, requires_user_confirm: true },
        CapabilityGrant { capability: Capability::ManageTabs, allowed: true, requires_user_confirm: false },
        CapabilityGrant { capability: Capability::AccessHistory, allowed: false, requires_user_confirm: true },
        CapabilityGrant { capability: Capability::AccessBookmarks, allowed: false, requires_user_confirm: true },
        CapabilityGrant { capability: Capability::ModifySettings, allowed: false, requires_user_confirm: true },
    ]
}
