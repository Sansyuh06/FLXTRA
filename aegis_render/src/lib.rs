//! Aegis Render Engine - Display list generation and painting
pub mod display_list;
pub mod painter;
pub use display_list::{DisplayList, DisplayCommand};
pub use painter::Painter;
