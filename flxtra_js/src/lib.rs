//! JavaScript Runtime
//! 
//! Responsible for:
//! - JavaScript execution using deno_core (V8 via Rust)
//! - Exposing custom Web APIs (fetch, setTimeout, DOM manipulation)
//! - DOM bridge: JS calls map to Rust DOM tree WITHOUT exposing raw DOM
//! - Per-tab JS isolation (each tab has its own context)

use std::collections::HashMap;

pub struct JsRuntime {
    globals: HashMap<String, JsValue>,
}

#[derive(Debug, Clone)]
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(HashMap<String, JsValue>),
}

impl JsRuntime {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        
        // Add global APIs
        globals.insert("console".to_string(), JsValue::Object(HashMap::new()));
        globals.insert("document".to_string(), JsValue::Object(HashMap::new()));
        globals.insert("window".to_string(), JsValue::Object(HashMap::new()));

        Self { globals }
    }

    pub fn execute(&mut self, code: &str) -> Result<JsValue, String> {
        // Basic JavaScript evaluation
        // In production, this would use deno_core + V8
        
        if code.contains("console.log") {
            Ok(JsValue::Undefined)
        } else if let Ok(num) = code.parse::<f64>() {
            Ok(JsValue::Number(num))
        } else if code.starts_with('"') || code.starts_with('\'') {
            Ok(JsValue::String(code.trim_matches(|c| c == '"' || c == '\'').to_string()))
        } else {
            Err(format!("Unsupported code: {}", code))
        }
    }

    pub fn get_global(&self, name: &str) -> Option<&JsValue> {
        self.globals.get(name)
    }
}
