use serde::{Serialize, Deserialize};
use tracing::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct DOMItem {
    pub id: u32,
    pub tag: String,
    pub label: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentPlan {
    pub action: String, // click, type, scroll
    pub target: u32,
    pub value: Option<String>,
    pub description: String,
}

// AI Service - Ollama Integration
pub fn call_ai(prompt: &str, action: &str, context: &str) -> String {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .ok();
    
    let full_prompt = match action {
        "summarize" => format!("Summarize this webpage content in 3 concise bullet points:\n\n{}", prompt),
        "explain" => format!("Explain this webpage content in simple terms that a 12-year-old could understand:\n\n{}", prompt),
        "keypoints" => format!("Extract the 5 most important facts from this content as a numbered list:\n\n{}", prompt),
        "ask" => format!("Context from webpage:\n{}\n\nQuestion: {}\n\nAnswer the question based on the context above:", context, prompt),
        _ => format!("Analyze this content:\n\n{}", prompt),
    };
    
    // Try Ollama first (local)
    if let Some(ref c) = client {
        if let Ok(res) = c.post("http://localhost:11434/api/generate")
            .json(&serde_json::json!({
                "model": "mistral",
                "prompt": full_prompt,
                "stream": false
            }))
            .send() 
        {
            if let Ok(body) = res.json::<serde_json::Value>() {
                if let Some(response) = body["response"].as_str() {
                    return response.to_string();
                }
            }
        }
    }
    
    if action == "ask" {
        return "Unable to answer. Please ensure Ollama is running locally.".to_string();
    }
    
    // Fallback: Inform user about Ollama requirement
    let action_label = match action {
        "summarize" => "summarize",
        "explain" => "explain",
        "keypoints" => "extract key points from",
        _ => "analyze"
    };
    
    format!("⚠️ **AI Offline**\n\nI couldn't {} this page because Ollama isn't running.\n\n**To enable AI:**\n1. Install Ollama: https://ollama.ai\n2. Run: `ollama serve`\n3. Download a model: `ollama pull mistral`\n\nThen try again!", action_label)
}

// Agent Planner - Ollama ReAct
pub fn call_agent_planner(goal: &str, dom: &[DOMItem]) -> Option<AgentPlan> {
    let dom_desc = dom.iter()
        .take(50) // Limit context
        .map(|d| format!("[{}] {} '{}'", d.id, d.tag, d.label))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "Goal: \"{}\"\n\nVisible Interactive Elements:\n{}\n\nReturn the NEXT step as a JSON object with fields: action (click/type), target (id), value (optional), description (short reason). JSON ONLY.",
        goal, dom_desc
    );

    let client = reqwest::blocking::Client::new();
    if let Ok(res) = client.post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "mistral",
            "prompt": prompt,
            "stream": false,
            "format": "json"
        }))
        .send() 
    {
        if let Ok(body) = res.json::<serde_json::Value>() {
            if let Some(resp_str) = body["response"].as_str() {
                // Try parsing JSON
                if let Ok(plan) = serde_json::from_str::<AgentPlan>(resp_str) {
                    return Some(plan);
                } else {
                    error!("Failed to parse agent plan JSON: {}", resp_str);
                }
            }
        }
    }
    None
}
