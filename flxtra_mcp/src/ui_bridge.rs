//! UI bridge — Comet-style sidebar state machine and IPC events.

use crate::automation::{ActionType, AutomationPlan, TaskReport};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// StepState / StepProgressItem
// ---------------------------------------------------------------------------

/// Per-step execution state shown in the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepState {
    Pending,
    Running,
    Done,
    Failed,
}

/// One row in the sidebar's step-progress list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepProgressItem {
    pub action: ActionType,
    pub description: String,
    pub state: StepState,
}

impl StepProgressItem {
    pub fn new(action: ActionType, description: impl Into<String>) -> Self {
        Self {
            action,
            description: description.into(),
            state: StepState::Pending,
        }
    }

    /// Emoji icon derived from the action type and current state.
    pub fn icon(&self) -> &'static str {
        match self.state {
            StepState::Pending => "⏸️",
            StepState::Running => "🔄",
            StepState::Done => "✅",
            StepState::Failed => "❌",
        }
    }

    /// Human-readable label: "🌐 Navigate" or "👆 Click".
    pub fn action_label(&self) -> String {
        format!("{} {}", self.action.icon(), self.action.label())
    }
}

// ---------------------------------------------------------------------------
// PanelState
// ---------------------------------------------------------------------------

/// High-level state of the automation sidebar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelState {
    Idle,
    OptimizingPrompt,
    GeneratingPlan,
    Executing {
        current_step: usize,
        total_steps: usize,
    },
    Complete,
    Failed,
}

impl Default for PanelState {
    fn default() -> Self {
        Self::Idle
    }
}

// ---------------------------------------------------------------------------
// TaskHistoryEntry
// ---------------------------------------------------------------------------

/// A completed task stored in history (ring buffer, max 50).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistoryEntry {
    pub task: String,
    pub success: bool,
    pub steps_total: usize,
    pub steps_succeeded: usize,
    pub duration_ms: u64,
    pub timestamp: String,
}

impl From<&TaskReport> for TaskHistoryEntry {
    fn from(r: &TaskReport) -> Self {
        Self {
            task: r.task.clone(),
            success: r.success,
            steps_total: r.total_steps,
            steps_succeeded: r.succeeded,
            duration_ms: r.total_duration_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// PanelEvent
// ---------------------------------------------------------------------------

/// Events flowing between the sidebar UI and the automation controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelEvent {
    /// User typed a task and pressed Go.
    UserSubmitted { task: String },
    /// A single step in the plan completed.
    StepCompleted { index: usize, success: bool },
    /// Entire task finished.
    TaskFinished { report: TaskReport },
    /// Task failed with an error message.
    TaskFailed { error: String },
    /// User clicked Reset.
    Reset,
    /// Toggle panel visibility (Ctrl+Shift+A).
    TogglePanel,
}

// ---------------------------------------------------------------------------
// FlxtraAutomationPanel
// ---------------------------------------------------------------------------

const MAX_HISTORY: usize = 50;

/// Sidebar panel state — drives rendering of the Comet-style automation UI.
#[derive(Debug, Clone)]
pub struct FlxtraAutomationPanel {
    pub state: PanelState,
    pub visible: bool,
    pub steps: Vec<StepProgressItem>,
    pub history: Vec<TaskHistoryEntry>,
    pub active_plan: Option<AutomationPlan>,
    pub last_error: Option<String>,
}

impl Default for FlxtraAutomationPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl FlxtraAutomationPanel {
    pub fn new() -> Self {
        Self {
            state: PanelState::Idle,
            visible: false,
            steps: Vec::new(),
            history: Vec::new(),
            active_plan: None,
            last_error: None,
        }
    }

    /// Progress percentage (0–100).
    pub fn progress_percent(&self) -> u8 {
        match &self.state {
            PanelState::Idle => 0,
            PanelState::OptimizingPrompt => 10,
            PanelState::GeneratingPlan => 25,
            PanelState::Executing {
                current_step,
                total_steps,
            } => {
                if *total_steps == 0 {
                    25
                } else {
                    let exec_pct = (*current_step as f64 / *total_steps as f64) * 70.0;
                    (25.0 + exec_pct).min(95.0) as u8
                }
            }
            PanelState::Complete => 100,
            PanelState::Failed => 0,
        }
    }

    /// Short human-readable status for the UI header.
    pub fn status_message(&self) -> &'static str {
        match &self.state {
            PanelState::Idle => "Ready",
            PanelState::OptimizingPrompt => "Optimizing prompt…",
            PanelState::GeneratingPlan => "Generating plan…",
            PanelState::Executing { .. } => "Executing steps…",
            PanelState::Complete => "Complete ✓",
            PanelState::Failed => "Failed ✗",
        }
    }

    /// Whether the panel is currently busy (should disable input).
    pub fn is_busy(&self) -> bool {
        matches!(
            self.state,
            PanelState::OptimizingPrompt
                | PanelState::GeneratingPlan
                | PanelState::Executing { .. }
        )
    }

    /// Lyra's welcome / idle message.
    pub fn welcome_message(&self) -> &'static str {
        "Hello! I'm Lyra, your AI prompt optimizer. I transform vague requests into precise, effective prompts."
    }

    /// Keyboard shortcut descriptor.
    pub fn toggle_shortcut() -> &'static str {
        "Ctrl+Shift+A"
    }

    /// Handle an incoming `PanelEvent` and mutate state accordingly.
    pub fn dispatch(&mut self, event: PanelEvent) {
        match event {
            PanelEvent::UserSubmitted { task } => {
                self.state = PanelState::OptimizingPrompt;
                self.steps.clear();
                self.last_error = None;
                log::info!("Panel: user submitted task: {}", task);
            }
            PanelEvent::StepCompleted { index, success } => {
                if let Some(item) = self.steps.get_mut(index) {
                    item.state = if success {
                        StepState::Done
                    } else {
                        StepState::Failed
                    };
                }
                // Advance running indicator.
                let next = index + 1;
                if next < self.steps.len() {
                    self.steps[next].state = StepState::Running;
                    self.state = PanelState::Executing {
                        current_step: next,
                        total_steps: self.steps.len(),
                    };
                }
            }
            PanelEvent::TaskFinished { report } => {
                self.state = if report.success {
                    PanelState::Complete
                } else {
                    PanelState::Failed
                };
                // Push to history (ring buffer).
                if self.history.len() >= MAX_HISTORY {
                    self.history.remove(0);
                }
                self.history.push(TaskHistoryEntry::from(&report));
            }
            PanelEvent::TaskFailed { error } => {
                self.state = PanelState::Failed;
                self.last_error = Some(error);
            }
            PanelEvent::Reset => {
                self.state = PanelState::Idle;
                self.steps.clear();
                self.active_plan = None;
                self.last_error = None;
            }
            PanelEvent::TogglePanel => {
                self.visible = !self.visible;
            }
        }
    }

    /// Populate step progress items from a generated plan.
    pub fn load_plan(&mut self, plan: &AutomationPlan) {
        self.steps = plan
            .steps
            .iter()
            .map(|s| StepProgressItem::new(s.action, &s.description))
            .collect();
        if let Some(first) = self.steps.first_mut() {
            first.state = StepState::Running;
        }
        self.active_plan = Some(plan.clone());
        self.state = PanelState::Executing {
            current_step: 0,
            total_steps: plan.len(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welcome_message_exact() {
        let panel = FlxtraAutomationPanel::new();
        assert_eq!(
            panel.welcome_message(),
            "Hello! I'm Lyra, your AI prompt optimizer. I transform vague requests into precise, effective prompts."
        );
    }

    #[test]
    fn progress_percent_states() {
        let mut p = FlxtraAutomationPanel::new();
        assert_eq!(p.progress_percent(), 0);

        p.state = PanelState::OptimizingPrompt;
        assert_eq!(p.progress_percent(), 10);

        p.state = PanelState::GeneratingPlan;
        assert_eq!(p.progress_percent(), 25);

        p.state = PanelState::Executing {
            current_step: 1,
            total_steps: 2,
        };
        assert_eq!(p.progress_percent(), 60); // 25 + (1/2)*70 = 60

        p.state = PanelState::Complete;
        assert_eq!(p.progress_percent(), 100);

        p.state = PanelState::Failed;
        assert_eq!(p.progress_percent(), 0);
    }

    #[test]
    fn dispatch_toggle() {
        let mut p = FlxtraAutomationPanel::new();
        assert!(!p.visible);
        p.dispatch(PanelEvent::TogglePanel);
        assert!(p.visible);
        p.dispatch(PanelEvent::TogglePanel);
        assert!(!p.visible);
    }

    #[test]
    fn dispatch_submit_and_reset() {
        let mut p = FlxtraAutomationPanel::new();
        p.dispatch(PanelEvent::UserSubmitted {
            task: "test".into(),
        });
        assert_eq!(p.state, PanelState::OptimizingPrompt);
        assert!(p.is_busy());

        p.dispatch(PanelEvent::Reset);
        assert_eq!(p.state, PanelState::Idle);
        assert!(!p.is_busy());
    }

    #[test]
    fn step_progress_icons() {
        let mut item = StepProgressItem::new(ActionType::Click, "click button");
        assert_eq!(item.icon(), "⏸️");
        item.state = StepState::Running;
        assert_eq!(item.icon(), "🔄");
        item.state = StepState::Done;
        assert_eq!(item.icon(), "✅");
        item.state = StepState::Failed;
        assert_eq!(item.icon(), "❌");
    }

    #[test]
    fn history_cap_at_50() {
        let mut p = FlxtraAutomationPanel::new();
        for i in 0..60 {
            let report = TaskReport {
                task: format!("task {}", i),
                total_steps: 1,
                succeeded: 1,
                failed: 0,
                skipped: 0,
                total_duration_ms: 100,
                step_results: vec![],
                success: true,
            };
            p.dispatch(PanelEvent::TaskFinished { report });
        }
        assert_eq!(p.history.len(), 50);
    }
}
