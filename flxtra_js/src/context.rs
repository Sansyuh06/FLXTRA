//! JavaScript execution context

use crate::value::{JsObject, JsValue};
use parking_lot::RwLock;
use std::sync::Arc;

/// Execution context for JavaScript
pub struct ExecutionContext {
    pub global: Arc<RwLock<JsObject>>,
    scopes: Vec<Arc<RwLock<JsObject>>>,
    pub this_value: JsValue,
}

impl ExecutionContext {
    pub fn new() -> Self {
        let global = Arc::new(RwLock::new(JsObject::new()));
        Self {
            global: global.clone(),
            scopes: vec![global],
            this_value: JsValue::Undefined,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Arc::new(RwLock::new(JsObject::new())));
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

    pub fn set(&self, name: &str, value: JsValue) {
        if let Some(scope) = self.scopes.last() { scope.write().set(name, value); }
    }

    pub fn declare(&self, name: &str, value: JsValue) { self.set(name, value); }

    pub fn assign(&self, name: &str, value: JsValue) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.read().has(name) { scope.write().set(name, value); return true; }
        }
        self.global.write().set(name, value);
        true
    }

    pub fn set_global(&self, name: &str, value: JsValue) {
        self.global.write().set(name, value);
    }

    pub fn get_global(&self, name: &str) -> JsValue {
        self.global.read().get(name)
    }
}

impl Default for ExecutionContext {
    fn default() -> Self { Self::new() }
}
