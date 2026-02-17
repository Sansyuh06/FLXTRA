//! Integration tests for flxtra_mcp using a MockBrowserBridge.

use async_trait::async_trait;
use flxtra_mcp::automation::*;
use flxtra_mcp::lyra::*;
use flxtra_mcp::ui_bridge::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ---------------------------------------------------------------------------
// MockBrowserBridge
// ---------------------------------------------------------------------------

/// In-memory mock that records every action for assertions.
#[derive(Debug, Clone, Default)]
struct MockState {
    url: String,
    clicks: Vec<String>,
    typed: Vec<(String, String)>,
    extracted: Vec<String>,
    evaluated: Vec<String>,
    keys_pressed: Vec<String>,
    uploads: Vec<(String, String)>,
}

struct MockBrowserBridge {
    state: Arc<Mutex<MockState>>,
    /// If set, `click` on this selector will return ElementNotFound.
    fail_selector: Option<String>,
}

impl MockBrowserBridge {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState {
                url: "about:blank".into(),
                ..Default::default()
            })),
            fail_selector: None,
        }
    }

    fn with_fail_selector(sel: &str) -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState {
                url: "about:blank".into(),
                ..Default::default()
            })),
            fail_selector: Some(sel.into()),
        }
    }

    fn state(&self) -> MockState {
        self.state.lock().unwrap().clone()
    }
}

#[async_trait]
impl BrowserBridge for MockBrowserBridge {
    async fn navigate(&self, url: &str) -> Result<(), BridgeError> {
        self.state.lock().unwrap().url = url.to_string();
        Ok(())
    }

    async fn click(&self, selector: &str) -> Result<(), BridgeError> {
        if Some(selector.to_string()) == self.fail_selector {
            return Err(BridgeError::ElementNotFound(selector.into()));
        }
        self.state.lock().unwrap().clicks.push(selector.into());
        Ok(())
    }

    async fn type_text(&self, selector: &str, text: &str) -> Result<(), BridgeError> {
        self.state
            .lock()
            .unwrap()
            .typed
            .push((selector.into(), text.into()));
        Ok(())
    }

    async fn extract_text(&self, selector: &str) -> Result<String, BridgeError> {
        self.state
            .lock()
            .unwrap()
            .extracted
            .push(selector.into());
        Ok(format!("text from {}", selector))
    }

    async fn wait_for_selector(&self, _selector: &str, _timeout: Duration) -> Result<(), BridgeError> {
        Ok(())
    }

    async fn evaluate(&self, script: &str) -> Result<String, BridgeError> {
        self.state.lock().unwrap().evaluated.push(script.into());
        Ok("ok".into())
    }

    async fn screenshot(&self) -> Result<Vec<u8>, BridgeError> {
        Ok(vec![0x89, 0x50, 0x4E, 0x47]) // PNG magic bytes
    }

    async fn current_url(&self) -> Result<String, BridgeError> {
        Ok(self.state.lock().unwrap().url.clone())
    }

    async fn press_key(&self, key: &str) -> Result<(), BridgeError> {
        self.state.lock().unwrap().keys_pressed.push(key.into());
        Ok(())
    }

    async fn upload_file(&self, selector: &str, path: &str) -> Result<(), BridgeError> {
        self.state
            .lock()
            .unwrap()
            .uploads
            .push((selector.into(), path.into()));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_navigation() {
    let bridge = MockBrowserBridge::new();
    let plan = AutomationPlan::new(
        "go to example",
        vec![AutomationStep::new(ActionType::Navigate)
            .value("https://example.com")
            .description("Navigate to example.com")],
    );
    let report = AutomationEngine::execute(&plan, &bridge).await.unwrap();
    assert!(report.success);
    assert_eq!(report.succeeded, 1);
    assert_eq!(bridge.state().url, "https://example.com");
}

#[tokio::test]
async fn test_multi_step_search() {
    let bridge = MockBrowserBridge::new();
    let plan = AutomationPlan::new(
        "search for Rust",
        vec![
            AutomationStep::new(ActionType::Navigate)
                .value("https://google.com")
                .description("Open Google"),
            AutomationStep::new(ActionType::TypeText)
                .selector("#search")
                .value("Rust programming")
                .description("Type query"),
            AutomationStep::new(ActionType::Click)
                .selector("#submit")
                .description("Click search"),
        ],
    );
    let report = AutomationEngine::execute(&plan, &bridge).await.unwrap();
    assert!(report.success);
    assert_eq!(report.total_steps, 3);
    assert_eq!(report.succeeded, 3);

    let s = bridge.state();
    assert_eq!(s.url, "https://google.com");
    assert_eq!(s.typed, vec![("#search".to_string(), "Rust programming".to_string())]);
    assert_eq!(s.clicks, vec!["#submit".to_string()]);
}

#[tokio::test]
async fn test_form_fill() {
    let bridge = MockBrowserBridge::new();
    let plan = AutomationPlan::new(
        "fill contact form",
        vec![
            AutomationStep::new(ActionType::TypeText)
                .selector("#name")
                .value("Alice")
                .description("Name"),
            AutomationStep::new(ActionType::TypeText)
                .selector("#email")
                .value("alice@x.com")
                .description("Email"),
            AutomationStep::new(ActionType::Click)
                .selector("#submit")
                .description("Submit"),
        ],
    );
    let report = AutomationEngine::execute(&plan, &bridge).await.unwrap();
    assert!(report.success);
    let s = bridge.state();
    assert_eq!(s.typed.len(), 2);
    assert_eq!(s.clicks, vec!["#submit".to_string()]);
}

#[tokio::test]
async fn test_fallback_on_missing_selector() {
    // Primary selector "#btn" will fail; fallback "#button" should succeed.
    let bridge = MockBrowserBridge::with_fail_selector("#btn");
    let plan = AutomationPlan::new(
        "click with fallback",
        vec![AutomationStep::new(ActionType::Click)
            .selector("#btn")
            .fallback("#button")
            .description("Click button")],
    );
    let report = AutomationEngine::execute(&plan, &bridge).await.unwrap();
    assert!(report.success);
    assert_eq!(bridge.state().clicks, vec!["#button".to_string()]);
}

#[tokio::test]
async fn test_continue_on_error() {
    let bridge = MockBrowserBridge::with_fail_selector("#missing");
    let plan = AutomationPlan::new(
        "resilient plan",
        vec![
            AutomationStep::new(ActionType::Click)
                .selector("#missing")
                .description("Will fail")
                .continue_on_err(true),
            AutomationStep::new(ActionType::Navigate)
                .value("https://x.com")
                .description("Should still run"),
        ],
    );
    let report = AutomationEngine::execute(&plan, &bridge).await.unwrap();
    // First step fails, second succeeds.
    assert_eq!(report.failed, 1);
    assert_eq!(report.succeeded, 1);
    assert!(!report.success); // Overall not 100% success.
    assert_eq!(bridge.state().url, "https://x.com");
}

#[tokio::test]
async fn test_stop_on_critical_failure() {
    let bridge = MockBrowserBridge::with_fail_selector("#missing");
    let plan = AutomationPlan::new(
        "stop on fail",
        vec![
            AutomationStep::new(ActionType::Click)
                .selector("#missing")
                .description("Critical step — no continue_on_error"),
            AutomationStep::new(ActionType::Navigate)
                .value("https://x.com")
                .description("Should be skipped"),
        ],
    );
    let report = AutomationEngine::execute(&plan, &bridge).await.unwrap();
    assert_eq!(report.failed, 1);
    assert_eq!(report.skipped, 1);
    assert_eq!(report.succeeded, 0);
}

#[test]
fn test_all_lyra_prompts() {
    let ctx = PageContext {
        url: "https://example.com".into(),
        title: "Test Page".into(),
        body_text: "Hello world".into(),
        form_fields: vec![FormField {
            label: "Email".into(),
            input_type: "email".into(),
            selector: "#email".into(),
            current_value: String::new(),
        }],
        interactive_elements: vec![InteractiveElement {
            tag: "button".into(),
            text: "Submit".into(),
            selector: "#submit".into(),
            role: "button".into(),
        }],
    };

    let auto = LyraPromptBuilder::build_automation_prompt("do task", &ctx);
    assert!(auto.contains("do task"));
    assert!(auto.contains("https://example.com"));
    assert!(auto.contains("Submit"));
    assert!(auto.contains("Email"));

    let ext = LyraPromptBuilder::build_extraction_prompt("get title", &ctx);
    assert!(ext.contains("get title"));

    let form = LyraPromptBuilder::build_form_prompt("fill it", &ctx);
    assert!(form.contains("Email"));
    assert!(form.contains("#email"));

    let opt_d = LyraPromptBuilder::build_optimization_prompt("vague", OptimizationMode::Detail);
    assert!(opt_d.contains("4-D"));

    let opt_b = LyraPromptBuilder::build_optimization_prompt("vague", OptimizationMode::Basic);
    assert!(opt_b.contains("Quick"));
}

#[test]
fn test_all_panel_events() {
    let mut panel = FlxtraAutomationPanel::new();

    // Toggle
    panel.dispatch(PanelEvent::TogglePanel);
    assert!(panel.visible);

    // Submit
    panel.dispatch(PanelEvent::UserSubmitted { task: "test".into() });
    assert_eq!(panel.state, PanelState::OptimizingPrompt);
    assert!(panel.is_busy());

    // Load a plan to get steps.
    let plan = AutomationPlan::new(
        "test",
        vec![
            AutomationStep::new(ActionType::Click).description("step 0"),
            AutomationStep::new(ActionType::TypeText).description("step 1"),
        ],
    );
    panel.load_plan(&plan);
    assert_eq!(
        panel.state,
        PanelState::Executing {
            current_step: 0,
            total_steps: 2
        }
    );

    // Step completed
    panel.dispatch(PanelEvent::StepCompleted { index: 0, success: true });
    assert_eq!(panel.steps[0].state, StepState::Done);
    assert_eq!(panel.steps[1].state, StepState::Running);

    // Task finished
    let report = TaskReport {
        task: "test".into(),
        total_steps: 2,
        succeeded: 2,
        failed: 0,
        skipped: 0,
        total_duration_ms: 42,
        step_results: vec![],
        success: true,
    };
    panel.dispatch(PanelEvent::TaskFinished { report });
    assert_eq!(panel.state, PanelState::Complete);
    assert_eq!(panel.history.len(), 1);

    // Task failed
    panel.dispatch(PanelEvent::TaskFailed { error: "oops".into() });
    assert_eq!(panel.state, PanelState::Failed);
    assert_eq!(panel.last_error.as_deref(), Some("oops"));

    // Reset
    panel.dispatch(PanelEvent::Reset);
    assert_eq!(panel.state, PanelState::Idle);
    assert!(panel.last_error.is_none());
}

#[test]
fn test_progress_percent_at_each_state() {
    let mut panel = FlxtraAutomationPanel::new();
    assert_eq!(panel.progress_percent(), 0); // Idle

    panel.state = PanelState::OptimizingPrompt;
    assert_eq!(panel.progress_percent(), 10);

    panel.state = PanelState::GeneratingPlan;
    assert_eq!(panel.progress_percent(), 25);

    panel.state = PanelState::Executing { current_step: 0, total_steps: 4 };
    assert_eq!(panel.progress_percent(), 25); // 25 + 0/4*70

    panel.state = PanelState::Executing { current_step: 2, total_steps: 4 };
    assert_eq!(panel.progress_percent(), 60); // 25 + 2/4*70 = 60

    panel.state = PanelState::Executing { current_step: 4, total_steps: 4 };
    assert_eq!(panel.progress_percent(), 95); // capped at 95

    panel.state = PanelState::Complete;
    assert_eq!(panel.progress_percent(), 100);

    panel.state = PanelState::Failed;
    assert_eq!(panel.progress_percent(), 0);
}

#[test]
fn test_empty_plan_error() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let bridge = MockBrowserBridge::new();
        let plan = AutomationPlan::new("empty", vec![]);
        let result = AutomationEngine::execute(&plan, &bridge).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AutomationError::EmptyPlan));
    });
}

#[test]
fn test_parse_plan_json_with_fences() {
    let raw = "```json\n[{\"action\":\"navigate\",\"value\":\"https://x.com\",\"description\":\"go\"}]\n```";
    let steps = flxtra_mcp::automation::parse_plan_json(raw).unwrap();
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].action, ActionType::Navigate);
}

#[test]
fn test_task_report_success_rate() {
    let r = TaskReport {
        task: "t".into(),
        total_steps: 4,
        succeeded: 3,
        failed: 1,
        skipped: 0,
        total_duration_ms: 100,
        step_results: vec![],
        success: false,
    };
    assert!((r.success_rate() - 0.75).abs() < f64::EPSILON);
}
