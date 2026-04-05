//! Automation Actions
//!
//! Individual actions that can be executed by the automation engine

use serde::{Deserialize, Serialize};
use std::time::Duration;
use flxtra_core::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Click { x: i32, y: i32 },
    DoubleClick { x: i32, y: i32 },
    RightClick { x: i32, y: i32 },
    TypeText(String),
    PressKey(String),
    Scroll { direction: String, amount: i32 },
    Wait(Duration),
    Screenshot,
    FindElement(String),
    Navigate(String),
    FillForm,
    ExtractText,
    WaitForElement(String),
    Drag { from_x: i32, from_y: i32, to_x: i32, to_y: i32 },
    Hover { x: i32, y: i32 },
    SelectText { start_x: i32, start_y: i32, end_x: i32, end_y: i32 },
    Copy,
    Paste,
    SwitchTab,
    CloseTab,
    Refresh,
    GoBack,
    GoForward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub description: String,
    pub timeout: Option<Duration>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub screenshot_before: Option<Vec<u8>>,
    pub screenshot_after: Option<Vec<u8>>,
    pub execution_time: Duration,
}

impl Action {
    /// Create a new action
    pub fn new(action_type: ActionType, description: String) -> Self {
        Self {
            action_type,
            description,
            timeout: Some(Duration::from_secs(10)),
            retry_count: 0,
        }
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set retry count
    pub fn with_retry(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    /// Execute the action
    pub async fn execute(&self) -> Result<ActionResult> {
        let start_time = std::time::Instant::now();

        // Implementation will be in the automation engine
        // This is just the structure

        let execution_time = start_time.elapsed();

        Ok(ActionResult {
            success: true,
            output: Some("Action executed successfully".to_string()),
            error: None,
            screenshot_before: None,
            screenshot_after: None,
            execution_time,
        })
    }
}

/// Predefined action templates
pub mod templates {
    use super::*;

    pub fn click_element(selector: &str) -> Action {
        Action::new(
            ActionType::FindElement(selector.to_string()),
            format!("Click element: {}", selector),
        )
    }

    pub fn type_text(text: &str) -> Action {
        Action::new(
            ActionType::TypeText(text.to_string()),
            format!("Type text: {}", text),
        )
    }

    pub fn navigate_to(url: &str) -> Action {
        Action::new(
            ActionType::Navigate(url.to_string()),
            format!("Navigate to: {}", url),
        )
    }

    pub fn fill_form() -> Action {
        Action::new(
            ActionType::FillForm,
            "Fill out form with stored data".to_string(),
        )
    }

    pub fn extract_data() -> Action {
        Action::new(
            ActionType::ExtractText,
            "Extract text/data from page".to_string(),
        )
    }

    pub fn wait_for_element(selector: &str) -> Action {
        Action::new(
            ActionType::WaitForElement(selector.to_string()),
            format!("Wait for element: {}", selector),
        ).with_timeout(Duration::from_secs(30))
    }

    pub fn scroll_down(amount: i32) -> Action {
        Action::new(
            ActionType::Scroll {
                direction: "down".to_string(),
                amount,
            },
            format!("Scroll down {} clicks", amount),
        )
    }

    pub fn take_screenshot() -> Action {
        Action::new(
            ActionType::Screenshot,
            "Take screenshot".to_string(),
        )
    }

    pub fn press_enter() -> Action {
        Action::new(
            ActionType::PressKey("enter".to_string()),
            "Press Enter key".to_string(),
        )
    }

    pub fn select_all() -> Action {
        Action::new(
            ActionType::PressKey("ctrl+a".to_string()),
            "Select all (Ctrl+A)".to_string(),
        )
    }

    pub fn copy() -> Action {
        Action::new(
            ActionType::Copy,
            "Copy to clipboard".to_string(),
        )
    }

    pub fn paste() -> Action {
        Action::new(
            ActionType::Paste,
            "Paste from clipboard".to_string(),
        )
    }
}