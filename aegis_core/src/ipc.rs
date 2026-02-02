//! Inter-process communication types for sandboxed components

use serde::{Deserialize, Serialize};
use crate::types::*;

/// Messages sent from browser process to renderer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserToRenderer {
    /// Navigate to a URL
    Navigate { url: String },
    /// Execute JavaScript
    ExecuteScript { script: String },
    /// Stop loading
    Stop,
    /// Reload page
    Reload,
    /// Go back
    GoBack,
    /// Go forward  
    GoForward,
    /// Resize viewport
    Resize { width: u32, height: u32 },
    /// Mouse event
    MouseEvent(MouseEvent),
    /// Keyboard event
    KeyEvent(KeyEvent),
    /// Shutdown
    Shutdown,
}

/// Messages sent from renderer to browser process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RendererToBrowser {
    /// Page started loading
    LoadStarted { url: String },
    /// Page finished loading
    LoadComplete { url: String },
    /// Page load failed
    LoadFailed { url: String, error: String },
    /// Title changed
    TitleChanged { title: String },
    /// Favicon updated
    FaviconChanged { url: Option<String> },
    /// Security state changed
    SecurityStateChanged(SecurityState),
    /// Console message
    ConsoleMessage { level: LogLevel, message: String },
    /// Request to navigate (link click, etc)
    NavigationRequest { url: String, is_new_tab: bool },
    /// Frame ready for display
    FrameReady { width: u32, height: u32 },
}

/// Messages sent from browser to network process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserToNetwork {
    /// Fetch a resource
    Fetch {
        request_id: u64,
        url: String,
        method: HttpMethod,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
        resource_type: ResourceType,
        origin: Option<Origin>,
    },
    /// Cancel a fetch
    CancelFetch { request_id: u64 },
    /// Resolve DNS
    ResolveDns { hostname: String },
    /// Clear cache
    ClearCache,
}

/// Messages sent from network to browser process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkToBrowser {
    /// Response headers received
    ResponseHeaders {
        request_id: u64,
        status: u16,
        headers: Vec<(String, String)>,
    },
    /// Response body chunk
    ResponseData {
        request_id: u64,
        data: Vec<u8>,
        is_final: bool,
    },
    /// Request blocked by filter
    RequestBlocked {
        request_id: u64,
        reason: String,
    },
    /// Request failed
    RequestFailed {
        request_id: u64,
        error: String,
    },
    /// DNS resolved
    DnsResolved {
        hostname: String,
        addresses: Vec<String>,
    },
}

/// Mouse event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub button: MouseButton,
    pub event_type: MouseEventType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MouseEventType {
    Move,
    Down,
    Up,
    Click,
    DoubleClick,
    Scroll { delta_x: f32, delta_y: f32 },
}

/// Keyboard event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key: String,
    pub code: String,
    pub event_type: KeyEventType,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyEventType {
    Down,
    Up,
    Press,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// Log level for console messages
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}
