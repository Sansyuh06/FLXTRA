//! FLXTRA Automation Engine
//!
//! Autonomous browser control system inspired by pyautogui
//! Features:
//! - Screen capture and analysis
//! - Mouse/keyboard automation
//! - OCR for text recognition
//! - Image matching for element detection
//! - Natural language command interpretation
//! - Multi-step workflow execution

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use parking_lot::RwLock;
use enigo::{Enigo, KeyboardControllable, MouseControllable};
use screenshots::Screen;
use image::DynamicImage;
use flxtra_core::{Result, FlxtraError};

pub mod actions;
pub mod vision;
pub mod workflows;
pub mod nlp;

pub use actions::{Action, ActionType, ActionResult};
pub use vision::{VisionEngine, ElementLocator, ScreenRegion};
pub use workflows::{Workflow, WorkflowStep, WorkflowEngine};
pub use nlp::{NLPParser, Intent, ParsedCommand};

/// Main automation engine
pub struct AutomationEngine {
    /// Input control (mouse/keyboard)
    enigo: Arc<Mutex<Enigo>>,
    /// Vision engine for screen analysis
    vision: VisionEngine,
    /// Workflow engine for multi-step tasks
    workflow_engine: WorkflowEngine,
    /// NLP parser for natural language commands
    nlp_parser: NLPParser,
    /// Active workflows
    _active_workflows: RwLock<HashMap<String, Workflow>>,
    /// Screen capture cache
    screen_cache: RwLock<Option<DynamicImage>>,
}

impl AutomationEngine {
    /// Create new automation engine
    pub async fn new() -> Result<Self> {
        let enigo = Arc::new(Mutex::new(Enigo::new()));
        let vision = VisionEngine::new().await?;
        let workflow_engine = WorkflowEngine::new();
        let nlp_parser = NLPParser::new();

        Ok(Self {
            enigo,
            vision,
            workflow_engine,
            nlp_parser,
            _active_workflows: RwLock::new(HashMap::new()),
            screen_cache: RwLock::new(None),
        })
    }

    /// Execute natural language command
    pub async fn execute_command(&self, command: &str) -> Result<String> {
        // Parse command into structured format
        let parsed = self.nlp_parser.parse_command(command)?;

        // Generate workflow from parsed command
        let workflow = self.nlp_parser.generate_workflow(&parsed)?;

        // Execute workflow
        let result = self.workflow_engine.execute_workflow(&workflow).await?;

        Ok(result)
    }

    /// Take screenshot and cache it
    pub async fn capture_screen(&self) -> Result<()> {
        let screen = Screen::from_point(0, 0)?;
        let image = screen.capture()?;
        // Convert screenshots::Image to image::DynamicImage
        let buffer = image.buffer();
        let rgba_image = image::RgbaImage::from_raw(
            image.width() as u32,
            image.height() as u32,
            buffer.to_vec(),
        ).ok_or_else(|| FlxtraError::Other(anyhow::anyhow!("Failed to convert screenshot")))?;
        let dynamic_image = DynamicImage::ImageRgba8(rgba_image);

        *self.screen_cache.write() = Some(dynamic_image);
        Ok(())
    }

    /// Get cached screenshot
    pub fn get_cached_screen(&self) -> Option<DynamicImage> {
        self.screen_cache.read().clone()
    }

    /// Find element on screen by text
    pub async fn find_element_by_text(&self, text: &str) -> Result<(i32, i32)> {
        self.vision.locate_text(text).await
    }

    /// Click at coordinates
    pub async fn click_at(&self, x: i32, y: i32) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        enigo.mouse_move_to(x, y);
        tokio::time::sleep(Duration::from_millis(100)).await;
        enigo.mouse_click(enigo::MouseButton::Left);
        Ok(())
    }

    /// Type text
    pub async fn type_text(&self, text: &str) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        enigo.key_sequence(text);
        Ok(())
    }

    /// Press key
    pub async fn press_key(&self, key: enigo::Key) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        enigo.key_click(key);
        Ok(())
    }

    /// Scroll
    pub async fn scroll(&self, direction: &str, clicks: i32) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        match direction.to_lowercase().as_str() {
            "up" => enigo.mouse_scroll_y(clicks),
            "down" => enigo.mouse_scroll_y(-clicks),
            "left" => enigo.mouse_scroll_x(-clicks),
            "right" => enigo.mouse_scroll_x(clicks),
            _ => enigo.mouse_scroll_y(-clicks), // default to down
        }
        Ok(())
    }
}

/// Scroll direction
#[derive(Debug, Clone, Copy)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Key enumeration
#[derive(Debug, Clone, Copy)]
pub enum Key {
    Enter,
    Tab,
    Escape,
    Space,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

impl From<Key> for enigo::Key {
    fn from(key: Key) -> Self {
        match key {
            Key::Enter => enigo::Key::Return,
            Key::Tab => enigo::Key::Tab,
            Key::Escape => enigo::Key::Escape,
            Key::Space => enigo::Key::Space,
            Key::Backspace => enigo::Key::Backspace,
            Key::Delete => enigo::Key::Delete,
            Key::Home => enigo::Key::Home,
            Key::End => enigo::Key::End,
            Key::PageUp => enigo::Key::PageUp,
            Key::PageDown => enigo::Key::PageDown,
            Key::ArrowUp => enigo::Key::UpArrow,
            Key::ArrowDown => enigo::Key::DownArrow,
            Key::ArrowLeft => enigo::Key::LeftArrow,
            Key::ArrowRight => enigo::Key::RightArrow,
        }
    }
}