//! JavaScript execution context with security hardening

use crate::value::{JsObject, JsValue};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::warn;

/// Protected property names that cannot be set (prevent prototype pollution)
const PROTECTED_PROPERTIES: &[&str] = &[
    "__proto__",
    "prototype", 
    "constructor",
    "__defineGetter__",
    "__defineSetter__",
    "__lookupGetter__",
    "__lookupSetter__",
];

/// Maximum scope depth to prevent stack overflow attacks
const MAX_SCOPE_DEPTH: usize = 512;

/// Execution context for JavaScript
pub struct ExecutionContext {
    pub global: Arc<RwLock<JsObject>>,
    scopes: Vec<Arc<RwLock<JsObject>>>,
    pub this_value: JsValue,
    strict_mode: bool,
}

impl ExecutionContext {
    pub fn new() -> Self {
        let global = Arc::new(RwLock::new(JsObject::new()));
        Self {
            global: global.clone(),
            scopes: vec![global],
            this_value: JsValue::Undefined,
            strict_mode: true, // Default to strict mode for security
        }
    }

    /// Check if a property name is protected against modification
    fn is_protected(name: &str) -> bool {
        PROTECTED_PROPERTIES.contains(&name)
    }

    pub fn enter_scope(&mut self) -> bool {
        if self.scopes.len() >= MAX_SCOPE_DEPTH {
            warn!("Maximum scope depth exceeded");
            return false;
        }
        self.scopes.push(Arc::new(RwLock::new(JsObject::new())));
        true
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 { self.scopes.pop(); }
    }

    pub fn get(&self, name: &str) -> JsValue {
        for scope in self.scopes.iter().rev() {
            if scope.read().has(name) { return scope.read().get(name); }
        }
        JsValue::Undefined
    }

    pub fn set(&self, name: &str, value: JsValue) -> bool {
        // Security: Block prototype pollution
        if Self::is_protected(name) {
            warn!("Attempt to set protected property: {}", name);
            return false;
        }
        
        if let Some(scope) = self.scopes.last() { 
            scope.write().set(name, value);
            true
        } else {
            false
        }
    }

    pub fn declare(&self, name: &str, value: JsValue) -> bool { 
        self.set(name, value) 
    }

    pub fn assign(&self, name: &str, value: JsValue) -> bool {
        // Security: Block prototype pollution
        if Self::is_protected(name) {
            warn!("Attempt to assign protected property: {}", name);
            return false;
        }
        
        for scope in self.scopes.iter().rev() {
            if scope.read().has(name) { 
                scope.write().set(name, value); 
                return true; 
            }
        }
        self.global.write().set(name, value);
        true
    }

    pub fn set_global(&self, name: &str, value: JsValue) -> bool {
        // Security: Block prototype pollution on globals
        if Self::is_protected(name) {
            warn!("Attempt to set protected global: {}", name);
            return false;
        }
        self.global.write().set(name, value);
        true
    }

    pub fn get_global(&self, name: &str) -> JsValue {
        self.global.read().get(name)
    }
    
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }
}

impl Default for ExecutionContext {
    fn default() -> Self { Self::new() }
}
