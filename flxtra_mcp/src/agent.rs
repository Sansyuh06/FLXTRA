//! LyraAgent — AI-powered plan generation via Anthropic-compatible API.
//!
//! Maintains per-tab conversation history and exposes methods for
//! plan generation, refinement, prompt optimization, and page analysis.

use crate::automation::{AutomationError, AutomationPlan, parse_plan_json};
use crate::lyra::{LyraPromptBuilder, OptimizationMode, PageContext, LYRA_SYSTEM_PROMPT};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Configuration for the AI backend.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// API endpoint (Anthropic-compatible).
    pub api_endpoint: String,
    /// API key — loaded from `FLXTRA_AI_KEY` env var by default.
    pub api_key: String,
    /// Model identifier.
    pub model: String,
    /// Max tokens for the completion.
    pub max_tokens: u32,
    /// Sampling temperature.
    pub temperature: f64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.anthropic.com/v1/messages".into(),
            api_key: std::env::var("FLXTRA_AI_KEY").unwrap_or_default(),
            model: "claude-sonnet-4-5-20250929".into(),
            max_tokens: 4096,
            temperature: 0.2,
        }
    }
}

// ---------------------------------------------------------------------------
// Conversation history
// ---------------------------------------------------------------------------

/// A single message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMessage {
    pub role: String,   // "user" | "assistant"
    pub content: String,
}

const MAX_HISTORY: usize = 20;

// ---------------------------------------------------------------------------
// LyraAgent
// ---------------------------------------------------------------------------

/// Orchestrates calls to the AI backend for a single browsing session.
#[derive(Debug, Clone)]
pub struct LyraAgent {
    config: AgentConfig,
    history: VecDeque<ApiMessage>,
    client: reqwest::Client,
}

impl LyraAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            history: VecDeque::with_capacity(MAX_HISTORY),
            client: reqwest::Client::new(),
        }
    }

    /// Clear conversation history.
    pub fn reset(&mut self) {
        self.history.clear();
    }

    // -- Public API ----------------------------------------------------------

    /// Generate a full automation plan from a natural-language task.
    pub async fn generate_plan(
        &mut self,
        task: &str,
        context: &PageContext,
    ) -> Result<AutomationPlan, AutomationError> {
        let user_prompt = LyraPromptBuilder::build_automation_prompt(task, context);
        let response = self
            .call_api(&user_prompt)
            .await
            .map_err(|e| AutomationError::PlanGeneration(e.to_string()))?;

        let steps = parse_plan_json(&response)?;
        Ok(AutomationPlan::new(task, steps))
    }

    /// Refine an existing plan based on execution feedback.
    pub async fn refine_plan(
        &mut self,
        original_task: &str,
        errors: &[String],
        context: &PageContext,
    ) -> Result<AutomationPlan, AutomationError> {
        let error_summary = errors.join("\n- ");
        let refine_prompt = format!(
            "The previous plan for \"{}\" had these errors:\n- {}\n\n\
             Current page state:\n- URL: {}\n- Title: {}\n\n\
             Generate a corrected JSON plan that avoids these errors.\n",
            original_task, error_summary, context.url, context.title,
        );
        let response = self
            .call_api(&refine_prompt)
            .await
            .map_err(|e| AutomationError::PlanGeneration(e.to_string()))?;

        let steps = parse_plan_json(&response)?;
        Ok(AutomationPlan::new(
            format!("{} (refined)", original_task),
            steps,
        ))
    }

    /// Rewrite / optimize a user's raw prompt using Lyra's 4-D methodology.
    pub async fn optimize_prompt(
        &mut self,
        raw: &str,
        mode: OptimizationMode,
    ) -> Result<String, AutomationError> {
        let prompt = LyraPromptBuilder::build_optimization_prompt(raw, mode);
        let response = self
            .call_api(&prompt)
            .await
            .map_err(|e| AutomationError::PlanGeneration(e.to_string()))?;
        Ok(response)
    }

    /// Analyze the current page and return a summary.
    pub async fn analyze_page(
        &mut self,
        context: &PageContext,
    ) -> Result<String, AutomationError> {
        let prompt = LyraPromptBuilder::build_extraction_prompt(
            "Provide a brief summary of this page's purpose, main content, and available actions.",
            context,
        );
        let response = self
            .call_api(&prompt)
            .await
            .map_err(|e| AutomationError::PlanGeneration(e.to_string()))?;
        Ok(response)
    }

    // -- Internal ------------------------------------------------------------

    /// Call the Anthropic-compatible messages API.
    async fn call_api(&mut self, user_message: &str) -> Result<String, AutomationError> {
        // Push user message into history.
        self.push_message("user", user_message);

        // Build the messages array for the API.
        let messages: Vec<serde_json::Value> = self
            .history
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "system": LYRA_SYSTEM_PROMPT,
            "messages": messages,
        });

        let response = self
            .client
            .post(&self.config.api_endpoint)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AutomationError::PlanGeneration(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(AutomationError::PlanGeneration(format!(
                "API returned {}: {}",
                status,
                body_text.chars().take(500).collect::<String>()
            )));
        }

        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AutomationError::PlanGeneration(format!("JSON decode: {}", e)))?;

        // Anthropic format: { "content": [{ "type": "text", "text": "..." }] }
        let assistant_text = resp_json["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .unwrap_or("")
            .to_string();

        // Push assistant reply into history.
        self.push_message("assistant", &assistant_text);

        Ok(assistant_text)
    }

    fn push_message(&mut self, role: &str, content: &str) {
        if self.history.len() >= MAX_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(ApiMessage {
            role: role.into(),
            content: content.into(),
        });
    }
}

// ---------------------------------------------------------------------------
// AgentSessionManager
// ---------------------------------------------------------------------------

/// Manages one `LyraAgent` per tab, keyed by `tab_id`.
#[derive(Debug)]
pub struct AgentSessionManager {
    sessions: HashMap<u64, LyraAgent>,
    config: AgentConfig,
}

impl AgentSessionManager {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            config,
        }
    }

    /// Get or create a session for the given tab.
    pub fn get_or_create(&mut self, tab_id: u64) -> &mut LyraAgent {
        self.sessions
            .entry(tab_id)
            .or_insert_with(|| LyraAgent::new(self.config.clone()))
    }

    /// Reset a specific tab's conversation context.
    pub fn reset_tab(&mut self, tab_id: u64) {
        if let Some(agent) = self.sessions.get_mut(&tab_id) {
            agent.reset();
        }
    }

    /// Remove a tab's session entirely.
    pub fn close_tab(&mut self, tab_id: u64) {
        self.sessions.remove(&tab_id);
    }

    /// Number of active sessions.
    pub fn active_sessions(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_manager_creates_and_closes() {
        let mut mgr = AgentSessionManager::new(AgentConfig::default());
        let _ = mgr.get_or_create(1);
        let _ = mgr.get_or_create(2);
        assert_eq!(mgr.active_sessions(), 2);
        mgr.close_tab(1);
        assert_eq!(mgr.active_sessions(), 1);
    }

    #[test]
    fn session_reset_clears_history() {
        let mut mgr = AgentSessionManager::new(AgentConfig::default());
        let agent = mgr.get_or_create(42);
        agent.push_message("user", "hello");
        assert!(!agent.history.is_empty());
        mgr.reset_tab(42);
        let agent = mgr.get_or_create(42);
        assert!(agent.history.is_empty());
    }

    #[test]
    fn config_defaults() {
        let cfg = AgentConfig::default();
        assert_eq!(cfg.model, "claude-sonnet-4-5-20250929");
        assert_eq!(cfg.max_tokens, 4096);
    }
}
