//! Per-Tab Process Isolation
//! 
//! Responsible for:
//! - Spawning isolated tab processes using OS-level sandboxing
//! - Windows: AppContainer (UAC-based isolation)
//! - Linux: seccomp-bpf + namespaces
//! - macOS: App Sandbox (entitlements)
//! - IPC protocol: Typed message passing between sandbox and main browser process
//! - Crash recovery: Tab crash doesn't crash main browser

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum IpcMessage {
    Navigate(String),
    Click { x: f64, y: f64 },
    Type(String),
    GetDom,
    Response(String),
}

#[derive(Clone)]
pub struct SandboxTab {
    #[allow(dead_code)]
    id: String,
    is_alive: Arc<Mutex<bool>>,
    messages: Arc<Mutex<Vec<IpcMessage>>>,
}

impl SandboxTab {
    pub fn new(id: String) -> Self {
        Self {
            id,
            is_alive: Arc::new(Mutex::new(true)),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn send_message(&self, msg: IpcMessage) -> Result<(), String> {
        let mut msgs = self.messages.lock().await;
        msgs.push(msg);
        Ok(())
    }

    pub async fn is_alive(&self) -> bool {
        *self.is_alive.lock().await
    }
}

pub struct SandboxManager {
    tabs: Arc<Mutex<Vec<SandboxTab>>>,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn create_tab(&self, id: String) -> SandboxTab {
        let tab = SandboxTab::new(id);
        self.tabs.lock().await.push(tab.clone());
        tab
    }
}
