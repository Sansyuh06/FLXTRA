use serde::{Serialize, Deserialize};
use tracing::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct DOMItem {
    pub id: u32,
    pub tag: String,
    pub label: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub r#type: String, // "text", "submit", etc.
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

// Agent Planner - Ollama ReAct with strict JSON output
pub fn call_agent_planner(goal: &str, dom: &[DOMItem]) -> Option<AgentPlan> {
    // Build a cleaner DOM representation
    let dom_desc = dom.iter()
        .take(50)
        .map(|d| {
            let type_str = if !d.r#type.is_empty() { format!(" type={}", d.r#type) } else { String::new() };
            let value_str = if !d.value.is_empty() { format!(" value=\"{}\"", d.value.chars().take(30).collect::<String>()) } else { String::new() };
            format!("[ID:{}] <{}{}{}> \"{}\"", d.id, d.tag, type_str, value_str, d.label.chars().take(40).collect::<String>())
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Check if this is a form-filling task
    let is_form_task = goal.to_lowercase().contains("fill") 
        || goal.to_lowercase().contains("form")
        || goal.to_lowercase().contains("enter")
        || goal.to_lowercase().contains("type");

    let prompt = if is_form_task {
        format!(
            r#"You are a browser automation agent. You must fill out a form on a webpage.

TASK: {}

AVAILABLE FORM ELEMENTS:
{}

RULES:
1. Return ONLY a JSON object, no other text
2. For input fields: {{"action": "type", "target": ID_NUMBER, "value": "text to type", "description": "what field"}}
3. For buttons/submit: {{"action": "click", "target": ID_NUMBER, "value": null, "description": "what button"}}
4. Pick the FIRST empty input field that matches the task
5. If the form looks complete, click the submit button

RESPOND WITH ONLY THE JSON OBJECT:"#,
            goal, dom_desc
        )
    } else {
        format!(
            r#"You are a browser automation agent. Execute ONE action to accomplish the goal.

GOAL: {}

VISIBLE ELEMENTS:
{}

Return ONLY a JSON object with: action (click/type/scroll), target (ID number), value (text for type, null for click), description (brief explanation).

JSON RESPONSE:"#,
            goal, dom_desc
        )
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .ok()?;
        
    let res = client.post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "mistral",
            "prompt": prompt,
            "stream": false,
            "format": "json",
            "options": {
                "temperature": 0.1,
                "num_predict": 200
            }
        }))
        .send()
        .ok()?;
    
    let body = res.json::<serde_json::Value>().ok()?;
    let resp_str = body["response"].as_str()?;
    
    tracing::info!("Agent raw response: {}", resp_str);
    
    // Try to parse the response directly
    if let Ok(plan) = serde_json::from_str::<AgentPlan>(resp_str) {
        return Some(plan);
    }
    
    // Try parsing as a wrapper with "actions" array
    if let Ok(wrapper) = serde_json::from_str::<serde_json::Value>(resp_str) {
        // Check for {"actions": [...]} format
        if let Some(actions) = wrapper.get("actions").and_then(|a| a.as_array()) {
            if let Some(first) = actions.first() {
                if let Ok(plan) = serde_json::from_value::<AgentPlan>(first.clone()) {
                    return Some(plan);
                }
            }
        }
        // Check for {"action": "...", ...} at root level
        if wrapper.get("action").is_some() {
            if let Ok(plan) = serde_json::from_value::<AgentPlan>(wrapper) {
                return Some(plan);
            }
        }
    }
    
    // Try to extract JSON from response if it has extra text
    if let Some(start) = resp_str.find('{') {
        if let Some(end) = resp_str.rfind('}') {
            let json_str = &resp_str[start..=end];
            if let Ok(plan) = serde_json::from_str::<AgentPlan>(json_str) {
                return Some(plan);
            }
        }
    }
    
    error!("Failed to parse agent plan: {}", resp_str);
    None
}
