//! Flxtra UI - Windows native browser shell
pub mod window;
pub mod events;
pub mod chrome;
pub mod pages;

pub use window::{BrowserWindow, WindowEvent};
pub use chrome::{BrowserChrome, ChromeHitResult, NavButton};
pub use pages::{render_new_tab_page, render_loading_page, render_error_page};
