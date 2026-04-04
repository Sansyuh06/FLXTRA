//! Agent Coordinator & Specialized Agents
//! 
//! Responsible for:
//! - Intent classification (user command -> agent type)
//! - Multi-agent orchestration (coordinator routes to right agent)
//! - Agent implementations:
//!   - Research Agent (web search + information gathering)
//!   - Automation Agent (form filling, task execution)
//!   - Summarization Agent (page -> summary via LLM)
//!   - Scraping Agent (extract structured data)
//!   - Security Agent (risk scoring, threat detection)
//! - Planning step: Agent generates numbered steps BEFORE execution
//! - Execution loop: Each step calls MCP tools, observes output
//! - Verification: Agent checks completion against original intent
//!
//! Security: Web content is DATA, never instructions (injection defense)
//! Memory: Coordinator tracks agent state, prevents partial claims of done

pub struct Coordinator;

impl Coordinator {
    pub fn new() -> Self {
        Self
    }
}

pub struct SummarizationAgent;
pub struct ResearchAgent;
pub struct AutomationAgent;
pub struct ScrapingAgent;
pub struct SecurityAgent;
