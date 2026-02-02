//! JavaScript values

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use parking_lot::RwLock;

/// JavaScript value type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    #[serde(skip)]
    Object(Arc<RwLock<JsObject>>),
    #[serde(skip)]
    Array(Arc<RwLock<Vec<JsValue>>>),
    #[serde(skip)]
    Function(Arc<JsFunction>),
    Symbol(String),
}

impl JsValue {
    pub fn undefined() -> Self {
        JsValue::Undefined
    }

    pub fn null() -> Self {
        JsValue::Null
    }

    pub fn boolean(v: bool) -> Self {
        JsValue::Boolean(v)
    }

    pub fn number(v: f64) -> Self {
        JsValue::Number(v)
    }

    pub fn string(v: &str) -> Self {
        JsValue::String(v.to_string())
    }

    pub fn object() -> Self {
        JsValue::Object(Arc::new(RwLock::new(JsObject::new())))
    }

    pub fn array(values: Vec<JsValue>) -> Self {
        JsValue::Array(Arc::new(RwLock::new(values)))
    }

    /// Type coercion to boolean
    pub fn to_boolean(&self) -> bool {
        match self {
            JsValue::Undefined | JsValue::Null => false,
            JsValue::Boolean(b) => *b,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::String(s) => !s.is_empty(),
            JsValue::Object(_) | JsValue::Array(_) | JsValue::Function(_) => true,
            JsValue::Symbol(_) => true,
        }
    }

    /// Type coercion to number
    pub fn to_number(&self) -> f64 {
        match self {
            JsValue::Undefined => f64::NAN,
            JsValue::Null => 0.0,
            JsValue::Boolean(b) => if *b { 1.0 } else { 0.0 },
            JsValue::Number(n) => *n,
            JsValue::String(s) => s.parse().unwrap_or(f64::NAN),
            JsValue::Object(_) | JsValue::Array(_) => f64::NAN,
            JsValue::Function(_) => f64::NAN,
            JsValue::Symbol(_) => f64::NAN,
        }
    }

    /// Type coercion to string
    pub fn to_string(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Boolean(b) => b.to_string(),
            JsValue::Number(n) => {
                if n.is_nan() {
                    "NaN".to_string()
                } else if n.is_infinite() {
                    if *n > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
                } else {
                    n.to_string()
                }
            }
            JsValue::String(s) => s.clone(),
            JsValue::Object(_) => "[object Object]".to_string(),
            JsValue::Array(arr) => {
                let arr = arr.read();
                arr.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }
            JsValue::Function(_) => "[function]".to_string(),
            JsValue::Symbol(s) => format!("Symbol({})", s),
        }
    }

    /// Get type name
    pub fn type_of(&self) -> &'static str {
        match self {
            JsValue::Undefined => "undefined",
            JsValue::Null => "object", // Historical quirk
            JsValue::Boolean(_) => "boolean",
            JsValue::Number(_) => "number",
            JsValue::String(_) => "string",
            JsValue::Object(_) | JsValue::Array(_) => "object",
            JsValue::Function(_) => "function",
            JsValue::Symbol(_) => "symbol",
        }
    }

    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        self.to_boolean()
    }

    /// Check if value is falsy
    pub fn is_falsy(&self) -> bool {
        !self.to_boolean()
    }

    /// Strict equality (===)
    pub fn strict_equals(&self, other: &JsValue) -> bool {
        match (self, other) {
            (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Number(a), JsValue::Number(b)) => {
                if a.is_nan() || b.is_nan() {
                    false
                } else {
                    a == b
                }
            }
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (JsValue::Object(a), JsValue::Object(b)) => Arc::ptr_eq(a, b),
            (JsValue::Array(a), JsValue::Array(b)) => Arc::ptr_eq(a, b),
            (JsValue::Function(a), JsValue::Function(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Default for JsValue {
    fn default() -> Self {
        JsValue::Undefined
    }
}

impl fmt::Display for JsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// JavaScript object
#[derive(Debug, Default)]
pub struct JsObject {
    pub properties: HashMap<String, JsValue>,
    pub prototype: Option<Arc<RwLock<JsObject>>>,
}

impl JsObject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> JsValue {
        if let Some(value) = self.properties.get(key) {
            return value.clone();
        }
        if let Some(proto) = &self.prototype {
            return proto.read().get(key);
        }
        JsValue::Undefined
    }

    pub fn set(&mut self, key: &str, value: JsValue) {
        self.properties.insert(key.to_string(), value);
    }

    pub fn has(&self, key: &str) -> bool {
        self.properties.contains_key(key)
            || self
                .prototype
                .as_ref()
                .map(|p| p.read().has(key))
                .unwrap_or(false)
    }

    pub fn delete(&mut self, key: &str) -> bool {
        self.properties.remove(key).is_some()
    }

    pub fn keys(&self) -> Vec<String> {
        self.properties.keys().cloned().collect()
    }
}

/// JavaScript function
#[derive(Debug)]
pub struct JsFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: FunctionBody,
}

/// Function body type
#[derive(Debug)]
pub enum FunctionBody {
    /// Native Rust function
    Native(fn(Vec<JsValue>) -> JsValue),
    /// Interpreted bytecode (placeholder for actual bytecode)
    Interpreted(Vec<u8>),
}

impl JsFunction {
    pub fn native(name: &str, func: fn(Vec<JsValue>) -> JsValue) -> Self {
        Self {
            name: name.to_string(),
            params: Vec::new(),
            body: FunctionBody::Native(func),
        }
    }
}
