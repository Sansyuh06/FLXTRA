//! Minimal UI Shell
//! 
//! Responsible for:
//! - Command bar: Center screen, persistent. Accepts URLs, search, natural language.
//! - Content canvas: Full screen, no chrome clutter
//! - Trust score badge: Real-time site analysis (green/yellow/red + number)
//! - Agent status strip: Live updates while agent is active
//! - Memory panel: Keyboard shortcut to open, view/edit/delete all items
//! - Principle: If explaining to new user takes >1 sentence, UI is too complex
//!
//! Design inspiration: Arc Browser (minimal), Comet (focus on AI)
//! The UI gets out of the way. The user types what they want. Browser figures it out.

pub struct CommandBar;
pub struct TrustBadge;
pub struct AgentStrip;
pub struct MemoryPanel;
