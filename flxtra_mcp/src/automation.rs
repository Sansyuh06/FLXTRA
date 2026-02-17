//! Automation engine — action types, plan execution, and the `BrowserBridge` trait.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors originating from the automation engine itself.
#[derive(Error, Debug)]
pub enum AutomationError {
    #[error("Bridge error: {0}")]
    Bridge(#[from] BridgeError),

    #[error("Plan is empty — nothing to execute")]
    EmptyPlan,

    #[error("Step {step} failed: {reason}")]
    StepFailed { step: usize, reason: String },

    #[error("Plan generation failed: {0}")]
    PlanGeneration(String),

    #[error("JSON parse error: {0}")]
    JsonParse(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}

/// Errors originating from the browser bridge implementation.
#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Navigation failed: {0}")]
    Navigation(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Script evaluation error: {0}")]
    ScriptError(String),

    #[error("Timeout waiting for selector: {0}")]
    SelectorTimeout(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Screenshot capture failed: {0}")]
    ScreenshotFailed(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("{0}")]
    Other(String),
}

// ---------------------------------------------------------------------------
// ActionType
// ---------------------------------------------------------------------------

/// Every atomic browser action the engine can dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Navigate,
    Click,
    TypeText,
    Scroll,
    Extract,
    Wait,
    Screenshot,
    FillForm,
    Evaluate,
    HoverOver,
    SelectOption,
    UploadFile,
    PressKey,
    GoBack,
    GoForward,
    Reload,
}

impl ActionType {
    /// Human-readable label for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Navigate => "Navigate",
            Self::Click => "Click",
            Self::TypeText => "Type",
            Self::Scroll => "Scroll",
            Self::Extract => "Extract",
            Self::Wait => "Wait",
            Self::Screenshot => "Screenshot",
            Self::FillForm => "Fill Form",
            Self::Evaluate => "Evaluate JS",
            Self::HoverOver => "Hover",
            Self::SelectOption => "Select",
            Self::UploadFile => "Upload",
            Self::PressKey => "Key Press",
            Self::GoBack => "Back",
            Self::GoForward => "Forward",
            Self::Reload => "Reload",
        }
    }

    /// Emoji icon for UI.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Navigate => "🌐",
            Self::Click => "👆",
            Self::TypeText => "⌨️",
            Self::Scroll => "📜",
            Self::Extract => "📋",
            Self::Wait => "⏳",
            Self::Screenshot => "📸",
            Self::FillForm => "📝",
            Self::Evaluate => "⚡",
            Self::HoverOver => "🖱️",
            Self::SelectOption => "☑️",
            Self::UploadFile => "📁",
            Self::PressKey => "🔑",
            Self::GoBack => "⬅️",
            Self::GoForward => "➡️",
            Self::Reload => "🔄",
        }
    }
}

// ---------------------------------------------------------------------------
// AutomationStep
// ---------------------------------------------------------------------------

/// A single step in an automation plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationStep {
    pub action: ActionType,
    #[serde(default)]
    pub selector: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub fallback_selector: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub continue_on_error: bool,
}

fn default_timeout() -> u64 {
    5000
}

impl AutomationStep {
    /// Create a new step with defaults.
    pub fn new(action: ActionType) -> Self {
        Self {
            action,
            selector: String::new(),
            value: None,
            description: String::new(),
            fallback_selector: None,
            timeout_ms: 5000,
            continue_on_error: false,
        }
    }

    // -- Builder methods --

    pub fn selector(mut self, s: impl Into<String>) -> Self {
        self.selector = s.into();
        self
    }

    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = Some(v.into());
        self
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }

    pub fn fallback(mut self, f: impl Into<String>) -> Self {
        self.fallback_selector = Some(f.into());
        self
    }

    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn continue_on_err(mut self, yes: bool) -> Self {
        self.continue_on_error = yes;
        self
    }
}

// ---------------------------------------------------------------------------
// AutomationPlan
// ---------------------------------------------------------------------------

/// An ordered sequence of steps produced by Lyra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationPlan {
    /// Human-readable task description.
    pub task: String,
    /// Ordered steps to execute.
    pub steps: Vec<AutomationStep>,
    /// When the plan was generated (ISO-8601).
    #[serde(default)]
    pub generated_at: String,
}

impl AutomationPlan {
    pub fn new(task: impl Into<String>, steps: Vec<AutomationStep>) -> Self {
        Self {
            task: task.into(),
            steps,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

// ---------------------------------------------------------------------------
// StepResult / TaskReport
// ---------------------------------------------------------------------------

/// Outcome of executing a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_index: usize,
    pub action: ActionType,
    pub success: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub extracted_data: Option<String>,
}

/// Aggregate report after executing an entire plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReport {
    pub task: String,
    pub total_steps: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_duration_ms: u64,
    pub step_results: Vec<StepResult>,
    pub success: bool,
}

impl TaskReport {
    pub fn success_rate(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        self.succeeded as f64 / self.total_steps as f64
    }
}

// ---------------------------------------------------------------------------
// BrowserBridge trait
// ---------------------------------------------------------------------------

/// Abstraction over the real browser (WebView2) or a mock for testing.
///
/// All methods are async and object-safe (`dyn BrowserBridge`).
#[async_trait]
pub trait BrowserBridge: Send + Sync {
    /// Navigate to a URL and wait for load.
    async fn navigate(&self, url: &str) -> Result<(), BridgeError>;

    /// Click on an element identified by CSS selector.
    async fn click(&self, selector: &str) -> Result<(), BridgeError>;

    /// Type text into an input identified by CSS selector.
    async fn type_text(&self, selector: &str, text: &str) -> Result<(), BridgeError>;

    /// Extract inner text of an element.
    async fn extract_text(&self, selector: &str) -> Result<String, BridgeError>;

    /// Wait until a selector appears in the DOM (up to `timeout`).
    async fn wait_for_selector(
        &self,
        selector: &str,
        timeout: Duration,
    ) -> Result<(), BridgeError>;

    /// Evaluate arbitrary JavaScript and return the result as a string.
    async fn evaluate(&self, script: &str) -> Result<String, BridgeError>;

    /// Capture a screenshot of the current viewport (PNG bytes).
    async fn screenshot(&self) -> Result<Vec<u8>, BridgeError>;

    /// Return the current page URL.
    async fn current_url(&self) -> Result<String, BridgeError>;

    /// Simulate a keyboard key press.
    async fn press_key(&self, key: &str) -> Result<(), BridgeError>;

    /// Upload a file to a `<input type="file">` element.
    async fn upload_file(&self, selector: &str, path: &str) -> Result<(), BridgeError>;
}

// ---------------------------------------------------------------------------
// AutomationEngine
// ---------------------------------------------------------------------------

/// Executes an `AutomationPlan` by dispatching each step to a `BrowserBridge`.
pub struct AutomationEngine;

impl AutomationEngine {
    /// Execute all steps in `plan` against `bridge`, collecting results.
    pub async fn execute(
        plan: &AutomationPlan,
        bridge: &dyn BrowserBridge,
    ) -> Result<TaskReport, AutomationError> {
        if plan.is_empty() {
            return Err(AutomationError::EmptyPlan);
        }

        let overall_start = Instant::now();
        let mut results: Vec<StepResult> = Vec::with_capacity(plan.len());
        let mut succeeded = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;
        let mut critical_failure = false;

        for (i, step) in plan.steps.iter().enumerate() {
            if critical_failure {
                skipped += 1;
                results.push(StepResult {
                    step_index: i,
                    action: step.action,
                    success: false,
                    duration_ms: 0,
                    error: Some("Skipped due to earlier failure".into()),
                    extracted_data: None,
                });
                continue;
            }

            let step_start = Instant::now();
            let outcome = Self::dispatch_step(step, bridge).await;
            let duration_ms = step_start.elapsed().as_millis() as u64;

            match outcome {
                Ok(data) => {
                    succeeded += 1;
                    results.push(StepResult {
                        step_index: i,
                        action: step.action,
                        success: true,
                        duration_ms,
                        error: None,
                        extracted_data: data,
                    });
                }
                Err(e) => {
                    failed += 1;
                    let err_msg = e.to_string();
                    results.push(StepResult {
                        step_index: i,
                        action: step.action,
                        success: false,
                        duration_ms,
                        error: Some(err_msg),
                        extracted_data: None,
                    });
                    if !step.continue_on_error {
                        critical_failure = true;
                    }
                }
            }
        }

        let total_duration_ms = overall_start.elapsed().as_millis() as u64;
        let success = failed == 0;

        Ok(TaskReport {
            task: plan.task.clone(),
            total_steps: plan.len(),
            succeeded,
            failed,
            skipped,
            total_duration_ms,
            step_results: results,
            success,
        })
    }

    /// Dispatch a single step, trying fallback selector on `ElementNotFound`.
    async fn dispatch_step(
        step: &AutomationStep,
        bridge: &dyn BrowserBridge,
    ) -> Result<Option<String>, BridgeError> {
        let result = Self::run_action(step, &step.selector, bridge).await;

        // If the primary selector fails with ElementNotFound and a fallback exists, retry.
        if let Err(BridgeError::ElementNotFound(_)) = &result {
            if let Some(ref fb) = step.fallback_selector {
                log::info!(
                    "Primary selector '{}' not found, trying fallback '{}'",
                    step.selector,
                    fb
                );
                return Self::run_action(step, fb, bridge).await;
            }
        }
        result
    }

    async fn run_action(
        step: &AutomationStep,
        selector: &str,
        bridge: &dyn BrowserBridge,
    ) -> Result<Option<String>, BridgeError> {
        match step.action {
            ActionType::Navigate => {
                let url = step.value.as_deref().unwrap_or("");
                bridge.navigate(url).await?;
                Ok(None)
            }
            ActionType::Click => {
                bridge.click(selector).await?;
                Ok(None)
            }
            ActionType::TypeText => {
                let text = step.value.as_deref().unwrap_or("");
                bridge.type_text(selector, text).await?;
                Ok(None)
            }
            ActionType::Scroll => {
                let amount = step.value.as_deref().unwrap_or("500");
                let script = format!("window.scrollBy(0, {});", amount);
                bridge.evaluate(&script).await?;
                Ok(None)
            }
            ActionType::Extract => {
                let text = bridge.extract_text(selector).await?;
                Ok(Some(text))
            }
            ActionType::Wait => {
                let timeout = Duration::from_millis(step.timeout_ms);
                bridge.wait_for_selector(selector, timeout).await?;
                Ok(None)
            }
            ActionType::Screenshot => {
                let _bytes = bridge.screenshot().await?;
                Ok(Some("screenshot_captured".into()))
            }
            ActionType::FillForm => {
                // FillForm is a convenience alias — the plan should decompose
                // into individual type_text steps. If it arrives here, treat
                // it like type_text.
                let text = step.value.as_deref().unwrap_or("");
                bridge.type_text(selector, text).await?;
                Ok(None)
            }
            ActionType::Evaluate => {
                let script = step.value.as_deref().unwrap_or("");
                let result = bridge.evaluate(script).await?;
                Ok(Some(result))
            }
            ActionType::HoverOver => {
                let script = format!(
                    "document.querySelector('{}')?.dispatchEvent(new MouseEvent('mouseover', {{bubbles:true}}))",
                    selector.replace('\'', "\\'")
                );
                bridge.evaluate(&script).await?;
                Ok(None)
            }
            ActionType::SelectOption => {
                let val = step.value.as_deref().unwrap_or("");
                let script = format!(
                    "var s=document.querySelector('{}');if(s){{s.value='{}';s.dispatchEvent(new Event('change',{{bubbles:true}}))}}",
                    selector.replace('\'', "\\'"),
                    val.replace('\'', "\\'"),
                );
                bridge.evaluate(&script).await?;
                Ok(None)
            }
            ActionType::UploadFile => {
                let path = step.value.as_deref().unwrap_or("");
                bridge.upload_file(selector, path).await?;
                Ok(None)
            }
            ActionType::PressKey => {
                let key = step.value.as_deref().unwrap_or("Enter");
                bridge.press_key(key).await?;
                Ok(None)
            }
            ActionType::GoBack => {
                bridge.evaluate("history.back()").await?;
                Ok(None)
            }
            ActionType::GoForward => {
                bridge.evaluate("history.forward()").await?;
                Ok(None)
            }
            ActionType::Reload => {
                bridge.evaluate("location.reload()").await?;
                Ok(None)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// JSON plan parsing utilities
// ---------------------------------------------------------------------------

/// Strip markdown code-fence wrappers (` ```json ... ``` `) that LLMs love to add.
pub fn strip_markdown_fences(raw: &str) -> &str {
    let trimmed = raw.trim();
    // Strip leading ```json or ```
    let after_open = if let Some(rest) = trimmed.strip_prefix("```json") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest
    } else {
        trimmed
    };
    // Strip trailing ```
    let before_close = if let Some(rest) = after_open.trim().strip_suffix("```") {
        rest
    } else {
        after_open
    };
    before_close.trim()
}

/// Parse an AI response into a list of `AutomationStep`s.
pub fn parse_plan_json(raw: &str) -> Result<Vec<AutomationStep>, AutomationError> {
    let clean = strip_markdown_fences(raw);

    // Try parsing as array first, then as single object wrapped in array.
    if let Ok(steps) = serde_json::from_str::<Vec<AutomationStep>>(clean) {
        return Ok(steps);
    }
    if let Ok(step) = serde_json::from_str::<AutomationStep>(clean) {
        return Ok(vec![step]);
    }
    // Try extracting JSON from surrounding prose.
    if let Some(start) = clean.find('[') {
        if let Some(end) = clean.rfind(']') {
            let slice = &clean[start..=end];
            if let Ok(steps) = serde_json::from_str::<Vec<AutomationStep>>(slice) {
                return Ok(steps);
            }
        }
    }
    Err(AutomationError::JsonParse(format!(
        "Could not parse steps from: {}",
        &clean.chars().take(200).collect::<String>()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_fences_works() {
        let raw = "```json\n[{\"action\":\"navigate\"}]\n```";
        assert_eq!(strip_markdown_fences(raw), "[{\"action\":\"navigate\"}]");
    }

    #[test]
    fn parse_single_step() {
        let json = r##"{"action":"click","selector":"#btn","description":"click it"}"##;
        let steps = parse_plan_json(json).unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].action, ActionType::Click);
    }

    #[test]
    fn parse_array() {
        let json = r##"[{"action":"navigate","value":"https://x.com"},{"action":"click","selector":"#login"}]"##;
        let steps = parse_plan_json(json).unwrap();
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn step_builder_chain() {
        let s = AutomationStep::new(ActionType::TypeText)
            .selector("#email")
            .value("me@x.com")
            .description("fill email")
            .fallback("#input-email")
            .timeout(3000)
            .continue_on_err(true);
        assert_eq!(s.selector, "#email");
        assert_eq!(s.value.as_deref(), Some("me@x.com"));
        assert_eq!(s.fallback_selector.as_deref(), Some("#input-email"));
        assert_eq!(s.timeout_ms, 3000);
        assert!(s.continue_on_error);
    }
}
