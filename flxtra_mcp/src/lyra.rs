//! Lyra — AI prompt optimization specialist for browser automation.
//!
//! Implements the 4-D methodology (Deconstruct, Diagnose, Develop, Deliver)
//! to transform natural-language user tasks into precise automation plans.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// System Prompt
// ---------------------------------------------------------------------------

/// Lyra's core system prompt, injected as the `system` role in every API call.
pub const LYRA_SYSTEM_PROMPT: &str = r#"You are Lyra, an expert browser-automation AI embedded in FLXTRA Browser.

## Identity
You are a master-level prompt optimization specialist who uses the 4-D methodology:
1. **Deconstruct** – Break the user's task into atomic browser actions.
2. **Diagnose**    – Identify the page context, selectors, and preconditions.
3. **Develop**     – Assemble a step-by-step JSON plan.
4. **Deliver**     – Return ONLY a valid JSON array of steps. No prose outside the JSON block.

## Output Contract
Return a JSON array where each element has:
{
  "action": "<ActionType>",
  "selector": "<CSS selector or empty>",
  "value": "<text / url / key / null>",
  "description": "<one-line human reason>",
  "fallback_selector": "<optional alternative selector or null>",
  "timeout_ms": <milliseconds, default 5000>,
  "continue_on_error": <true|false, default false>
}

Valid ActionType values:
navigate, click, type_text, scroll, extract, wait, screenshot,
fill_form, evaluate, hover_over, select_option, upload_file,
press_key, go_back, go_forward, reload.

## Rules
- Never wrap the JSON in markdown fences.
- If the task is ambiguous, make a reasonable assumption and note it in `description`.
- For navigation, set `value` to the full URL.
- For form filling, break each field into a separate `type_text` step.
- Keep plans as short as possible — fewer steps is better.
"#;

// ---------------------------------------------------------------------------
// PageContext
// ---------------------------------------------------------------------------

/// Snapshot of the current page state provided to Lyra for context-aware planning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageContext {
    /// Current URL of the page.
    pub url: String,
    /// Document title.
    pub title: String,
    /// Truncated visible body text (first ~2000 chars).
    pub body_text: String,
    /// Detected form fields (label → input type).
    pub form_fields: Vec<FormField>,
    /// Clickable / interactive elements visible on screen.
    pub interactive_elements: Vec<InteractiveElement>,
}

/// A single form field detected on the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub label: String,
    pub input_type: String,
    pub selector: String,
    pub current_value: String,
}

/// A clickable or interactive element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveElement {
    pub tag: String,
    pub text: String,
    pub selector: String,
    pub role: String,
}

// ---------------------------------------------------------------------------
// OptimizationMode
// ---------------------------------------------------------------------------

/// Controls the verbosity and depth of Lyra's prompt rewriting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationMode {
    /// Full 4-D analysis with detailed reasoning.
    Detail,
    /// Quick rewrite — concise and direct.
    Basic,
}

impl Default for OptimizationMode {
    fn default() -> Self {
        Self::Basic
    }
}

// ---------------------------------------------------------------------------
// LyraPromptBuilder
// ---------------------------------------------------------------------------

/// Builds domain-specific prompts that are appended after the system prompt.
#[derive(Debug, Clone)]
pub struct LyraPromptBuilder;

impl LyraPromptBuilder {
    /// Build a prompt for general browser automation.
    ///
    /// Combines the user's task with the current page context so Lyra can
    /// produce a step-by-step action plan.
    pub fn build_automation_prompt(task: &str, context: &PageContext) -> String {
        let mut prompt = String::with_capacity(2048);
        prompt.push_str("## Current Page\n");
        prompt.push_str(&format!("- URL: {}\n", context.url));
        prompt.push_str(&format!("- Title: {}\n", context.title));

        if !context.interactive_elements.is_empty() {
            prompt.push_str("\n### Interactive Elements\n");
            for (i, el) in context.interactive_elements.iter().take(30).enumerate() {
                prompt.push_str(&format!(
                    "{}. <{}> \"{}\" selector=`{}` role={}\n",
                    i + 1,
                    el.tag,
                    el.text.chars().take(50).collect::<String>(),
                    el.selector,
                    el.role,
                ));
            }
        }

        if !context.form_fields.is_empty() {
            prompt.push_str("\n### Form Fields\n");
            for f in &context.form_fields {
                prompt.push_str(&format!(
                    "- \"{}\" type={} selector=`{}`{}\n",
                    f.label,
                    f.input_type,
                    f.selector,
                    if f.current_value.is_empty() {
                        String::new()
                    } else {
                        format!(" value=\"{}\"", f.current_value)
                    },
                ));
            }
        }

        if !context.body_text.is_empty() {
            prompt.push_str("\n### Page Text (truncated)\n");
            prompt.push_str(&context.body_text.chars().take(1500).collect::<String>());
            prompt.push('\n');
        }

        prompt.push_str(&format!("\n## Task\n{}\n", task));
        prompt.push_str("\nReturn the automation plan as a JSON array.\n");
        prompt
    }

    /// Build a prompt for extracting structured data from a page.
    pub fn build_extraction_prompt(what: &str, context: &PageContext) -> String {
        format!(
            "## Current Page\n- URL: {}\n- Title: {}\n\n\
             ### Page Text (truncated)\n{}\n\n\
             ## Extraction Task\nExtract: {}\n\n\
             Return a JSON object with the extracted data. \
             Use descriptive keys.\n",
            context.url,
            context.title,
            context.body_text.chars().take(2000).collect::<String>(),
            what,
        )
    }

    /// Build a prompt specifically for filling a form.
    pub fn build_form_prompt(
        instructions: &str,
        context: &PageContext,
    ) -> String {
        let mut prompt = String::with_capacity(1024);
        prompt.push_str("## Form Detected\n");

        for f in &context.form_fields {
            prompt.push_str(&format!(
                "- Field \"{}\" (type={}, selector=`{}`){}\n",
                f.label,
                f.input_type,
                f.selector,
                if f.current_value.is_empty() {
                    " [empty]".to_string()
                } else {
                    format!(" value=\"{}\"", f.current_value)
                },
            ));
        }

        prompt.push_str(&format!(
            "\n## Instructions\n{}\n\n\
             Generate a JSON array of `type_text` steps to fill this form, \
             followed by a `click` step on the submit button.\n",
            instructions,
        ));
        prompt
    }

    /// Build a prompt that rewrites/optimizes a user's raw prompt.
    pub fn build_optimization_prompt(raw_prompt: &str, mode: OptimizationMode) -> String {
        match mode {
            OptimizationMode::Detail => format!(
                "## Prompt Optimization (Detailed 4-D)\n\
                 Apply the full 4-D methodology to improve this prompt:\n\n\
                 > {}\n\n\
                 Return a JSON object: \
                 {{\"optimized\": \"<improved prompt>\", \
                 \"reasoning\": \"<brief 4-D analysis>\"}}\n",
                raw_prompt,
            ),
            OptimizationMode::Basic => format!(
                "## Quick Prompt Optimization\n\
                 Rewrite this prompt to be clearer and more specific:\n\n\
                 > {}\n\n\
                 Return a JSON object: {{\"optimized\": \"<improved prompt>\"}}\n",
                raw_prompt,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automation_prompt_includes_task_and_url() {
        let ctx = PageContext {
            url: "https://example.com".into(),
            title: "Example".into(),
            ..Default::default()
        };
        let p = LyraPromptBuilder::build_automation_prompt("click login", &ctx);
        assert!(p.contains("click login"));
        assert!(p.contains("https://example.com"));
    }

    #[test]
    fn form_prompt_lists_fields() {
        let ctx = PageContext {
            form_fields: vec![FormField {
                label: "Email".into(),
                input_type: "email".into(),
                selector: "#email".into(),
                current_value: String::new(),
            }],
            ..Default::default()
        };
        let p = LyraPromptBuilder::build_form_prompt("fill with test@x.com", &ctx);
        assert!(p.contains("Email"));
        assert!(p.contains("#email"));
    }

    #[test]
    fn optimization_prompt_modes() {
        let d = LyraPromptBuilder::build_optimization_prompt("do stuff", OptimizationMode::Detail);
        assert!(d.contains("4-D"));
        let b = LyraPromptBuilder::build_optimization_prompt("do stuff", OptimizationMode::Basic);
        assert!(b.contains("Quick"));
    }
}
