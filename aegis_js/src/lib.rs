//! Aegis JavaScript Engine
//!
//! Secure JavaScript interpreter with:
//! - No JIT by default (security)
//! - Timing jitter for fingerprint resistance
//! - Strict same-origin enforcement
//! - Sandboxed execution context

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod runtime;
pub mod value;
pub mod builtins;
pub mod context;

pub use runtime::JsRuntime;
pub use value::JsValue;
pub use context::ExecutionContext;
