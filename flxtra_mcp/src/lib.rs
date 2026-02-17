//! flxtra_mcp — Lyra AI-powered browser automation for FLXTRA.
//!
//! This crate provides a Comet-style agentic automation system that translates
//! natural-language user tasks into multi-step browser actions, executed via
//! the [`BrowserBridge`] trait.
//!
//! # Quick Start
//! ```rust,ignore
//! use flxtra_mcp::run_quick_task;
//!
//! let report = run_quick_task(bridge, 1, "search for Rust tutorials").await?;
//! println!("Success: {}", report.success);
//! ```

pub mod agent;
pub mod automation;
pub mod lyra;
pub mod ui_bridge;

// Re-exports for convenience.
pub use agent::{AgentConfig, AgentSessionManager, LyraAgent};
pub use automation::{
    ActionType, AutomationEngine, AutomationError, AutomationPlan, AutomationStep,
    BridgeError, BrowserBridge, StepResult, TaskReport,
};
pub use lyra::{LyraPromptBuilder, OptimizationMode, PageContext, LYRA_SYSTEM_PROMPT};
pub use ui_bridge::{FlxtraAutomationPanel, PanelEvent, PanelState, StepProgressItem};

use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// AutomationController
// ---------------------------------------------------------------------------

/// Top-level controller that wires together the agent, engine, and UI panel.
///
/// Thread-safe via `Arc<Mutex<>>` for access from both the UI thread and the
/// async runtime.
#[derive(Clone)]
pub struct AutomationController {
    inner: Arc<Mutex<ControllerInner>>,
}

struct ControllerInner {
    sessions: AgentSessionManager,
    panel: FlxtraAutomationPanel,
}

impl std::fmt::Debug for AutomationController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutomationController").finish()
    }
}

const MAX_RETRIES: usize = 2;

impl AutomationController {
    /// Create a new controller with the given agent config.
    pub fn new(config: AgentConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ControllerInner {
                sessions: AgentSessionManager::new(config),
                panel: FlxtraAutomationPanel::new(),
            })),
        }
    }

    /// Create with default config (reads `FLXTRA_AI_KEY` from env).
    pub fn with_defaults() -> Self {
        Self::new(AgentConfig::default())
    }

    /// Run a full automation task: optimize → plan → execute → auto-retry.
    pub async fn run_task(
        &self,
        bridge: &dyn BrowserBridge,
        tab_id: u64,
        task: &str,
        context: &PageContext,
    ) -> Result<TaskReport, AutomationError> {
        // 1. Update panel state.
        {
            let mut inner = self.inner.lock().await;
            inner.panel.dispatch(PanelEvent::UserSubmitted {
                task: task.into(),
            });
        }

        // 2. Generate plan.
        let plan = {
            let mut inner = self.inner.lock().await;
            inner.panel.state = PanelState::GeneratingPlan;
            let agent = inner.sessions.get_or_create(tab_id);
            agent.generate_plan(task, context).await?
        };

        // 3. Load plan into panel.
        {
            let mut inner = self.inner.lock().await;
            inner.panel.load_plan(&plan);
        }

        // 4. Execute.
        let mut report = AutomationEngine::execute(&plan, bridge).await?;

        // 5. Auto-retry on failure (up to MAX_RETRIES).
        let mut retries = 0;
        while !report.success && retries < MAX_RETRIES {
            retries += 1;
            log::info!("Auto-retry {}/{} for task: {}", retries, MAX_RETRIES, task);

            let errors: Vec<String> = report
                .step_results
                .iter()
                .filter_map(|r| r.error.clone())
                .collect();

            let refined_plan = {
                let mut inner = self.inner.lock().await;
                inner.panel.state = PanelState::GeneratingPlan;
                let agent = inner.sessions.get_or_create(tab_id);
                agent.refine_plan(task, &errors, context).await?
            };

            {
                let mut inner = self.inner.lock().await;
                inner.panel.load_plan(&refined_plan);
            }

            report = AutomationEngine::execute(&refined_plan, bridge).await?;
        }

        // 6. Finalize panel state.
        {
            let mut inner = self.inner.lock().await;
            inner
                .panel
                .dispatch(PanelEvent::TaskFinished { report: report.clone() });
        }

        Ok(report)
    }

    /// Optimize a user's prompt without executing.
    pub async fn optimize_prompt(
        &self,
        tab_id: u64,
        raw: &str,
        mode: OptimizationMode,
    ) -> Result<String, AutomationError> {
        let mut inner = self.inner.lock().await;
        let agent = inner.sessions.get_or_create(tab_id);
        agent.optimize_prompt(raw, mode).await
    }

    /// Analyze the current page.
    pub async fn analyze_page(
        &self,
        tab_id: u64,
        context: &PageContext,
    ) -> Result<String, AutomationError> {
        let mut inner = self.inner.lock().await;
        let agent = inner.sessions.get_or_create(tab_id);
        agent.analyze_page(context).await
    }

    /// Reset conversation context for a tab.
    pub async fn reset_tab_context(&self, tab_id: u64) {
        let mut inner = self.inner.lock().await;
        inner.sessions.reset_tab(tab_id);
    }

    /// Close a tab session.
    pub async fn close_tab(&self, tab_id: u64) {
        let mut inner = self.inner.lock().await;
        inner.sessions.close_tab(tab_id);
    }

    /// Get a snapshot of the current panel state.
    pub async fn panel_state(&self) -> PanelState {
        let inner = self.inner.lock().await;
        inner.panel.state.clone()
    }

    /// Dispatch a panel event.
    pub async fn dispatch_panel_event(&self, event: PanelEvent) {
        let mut inner = self.inner.lock().await;
        inner.panel.dispatch(event);
    }

    /// Get the welcome message.
    pub fn welcome_message(&self) -> &'static str {
        "Hello! I'm Lyra, your AI prompt optimizer. I transform vague requests into precise, effective prompts."
    }
}

// ---------------------------------------------------------------------------
// Convenience function
// ---------------------------------------------------------------------------

/// One-shot convenience: create a controller, run a task, return the report.
pub async fn run_quick_task(
    bridge: &dyn BrowserBridge,
    tab_id: u64,
    task: &str,
) -> Result<TaskReport, AutomationError> {
    let ctx = PageContext {
        url: bridge
            .current_url()
            .await
            .unwrap_or_else(|_| "about:blank".into()),
        ..Default::default()
    };
    let ctrl = AutomationController::with_defaults();
    ctrl.run_task(bridge, tab_id, task, &ctx).await
}
