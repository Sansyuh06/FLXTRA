//! Minimal UI Shell
//! 
//! Responsible for:
//! - Command bar: Center screen, persistent. Accepts URLs, search, natural language.
//! - Content canvas: Full screen, no chrome clutter
//! - Trust score badge: Real-time site analysis (green/yellow/red + number)
//! - Agent status strip: Live updates while agent is active
//! - Memory panel: Keyboard shortcut to open, view/edit/delete all items
//! - Principle: If explaining to new user takes >1 sentence, UI is too complex
//!
//! Design inspiration: Arc Browser (minimal), Comet (focus on AI)
//! The UI gets out of the way. The user types what they want. Browser figures it out.

use std::collections::HashMap;

pub struct CommandBar {
    pub input: String,
    pub suggestions: Vec<String>,
}

impl CommandBar {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn execute(&self) -> Option<String> {
        if self.input.is_empty() {
            None
        } else {
            Some(self.input.clone())
        }
    }
}

pub struct TrustBadge {
    pub score: u32,
    pub level: String,
}

impl TrustBadge {
    pub fn new() -> Self {
        Self {
            score: 100,
            level: "safe".to_string(),
        }
    }
}

pub struct AgentStrip {
    pub is_active: bool,
    pub current_action: String,
}

impl AgentStrip {
    pub fn new() -> Self {
        Self {
            is_active: false,
            current_action: String::new(),
        }
    }
}

pub struct MemoryPanel {
    pub items: HashMap<String, String>,
    pub is_visible: bool,
}

impl MemoryPanel {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            is_visible: false,
        }
    }
}

pub struct UiController {
    pub command_bar: CommandBar,
    pub trust_badge: TrustBadge,
    pub agent_strip: AgentStrip,
    pub memory_panel: MemoryPanel,
}

impl UiController {
    pub fn new() -> Self {
        Self {
            command_bar: CommandBar::new(),
            trust_badge: TrustBadge::new(),
            agent_strip: AgentStrip::new(),
            memory_panel: MemoryPanel::new(),
        }
    }
}
