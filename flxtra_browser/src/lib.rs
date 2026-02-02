//! Flxtra Browser - Privacy-first web browser
//!
//! Main entry point that orchestrates all browser components.

mod browser;
mod tab;
mod engine;

pub use browser::Browser;
pub use tab::Tab;
pub use engine::RenderEngine;
