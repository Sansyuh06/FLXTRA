//! Main Browser controller with full rendering and interactivity
use flxtra_core::{BrowserConfig, ContainerId, TabId, Result, FlxtraError};
use flxtra_css::values::Color;
use flxtra_filter::FilterEngine;
use flxtra_html::{HtmlParser, DomTree};
use flxtra_layout::{LayoutTree, LayoutEngine, LayoutBox};
use flxtra_mcp::McpClient;
use flxtra_net::NetworkClient;
use flxtra_render::{DisplayList, Painter, GdiRenderer};
use flxtra_sandbox::container::ContainerManager;
use flxtra_ui::{BrowserWindow, BrowserChrome, ChromeHitResult, NavButton, WindowEvent};
use flxtra_ui::{render_new_tab_page, render_loading_page, render_error_page};
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{info, debug, error};
use flxtra_core::ResourceType;

/// Page state for a tab
pub struct PageState {
    pub url: String,
    pub title: String,
    pub is_loading: bool,
    pub display_list: DisplayList,
    pub layout_root: Option<LayoutBox>,
    pub error: Option<String>,
}

impl PageState {
    pub fn new_tab() -> Self {
        Self {
            url: "flxtra://newtab".to_string(),
            title: "New Tab".to_string(),
            is_loading: false,
            display_list: DisplayList::new(),
            layout_root: None,
            error: None,
        }
    }
}

/// Full browser application
pub struct Browser {
    pub config: BrowserConfig,
    pages: HashMap<TabId, PageState>,
    chrome: BrowserChrome,
    network: Arc<NetworkClient>,
    filter: Arc<FilterEngine>,
    containers: ContainerManager,
    mcp: McpClient,
    renderer: Option<GdiRenderer>,
    window: Option<BrowserWindow>,
    active_tab: Option<TabId>,
}

impl Browser {
    pub async fn new(config: BrowserConfig) -> Result<Self> {
        info!("Initializing Flxtra Browser");

        let network = Arc::new(
            NetworkClient::new(
                &config.network.doh_server,
                config.network.allow_http,
                config.network.timeout_seconds as u64,
            )
            .await?,
        );

        let filter = Arc::new(FilterEngine::new());

        let mut containers = ContainerManager::new();
        containers.add(ContainerManager::default_container());
        containers.add(ContainerManager::banking_container());

        let width = config.ui.window_width as f32;
        let height = config.ui.window_height as f32;

        let mut pages = HashMap::new();
        let first_tab = TabId::new();
        pages.insert(first_tab, PageState::new_tab());

        Ok(Self {
            config,
            pages,
            chrome: BrowserChrome::new(width, height),
            network,
            filter,
            containers,
            mcp: McpClient::new(),
            renderer: None,
            window: None,
            active_tab: Some(first_tab),
        })
    }

    pub fn handle_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Click { x, y } => {
                self.handle_click(x, y);
            }
            WindowEvent::KeyPress { char: c } => {
                self.handle_key(c);
            }
            WindowEvent::Resize { width, height } => {
                self.chrome.resize(width as f32, height as f32);
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(width, height);
                }
            }
            WindowEvent::Paint => {
                let _ = self.render();
            }
            WindowEvent::Close => {
                info!("Browser closing");
            }
        }
    }

    pub fn handle_click(&mut self, x: f32, y: f32) {
        info!("Click at ({}, {})", x, y);
        
        if let Some(hit) = self.chrome.hit_test(x, y) {
            info!("Hit: {:?}", hit);
            match hit {
                ChromeHitResult::Tab(index) => {
                    self.chrome.switch_tab(index);
                    if let Some((tab_id, _)) = self.pages.iter().nth(index) {
                        self.active_tab = Some(*tab_id);
                    }
                }
                ChromeHitResult::NewTab => {
                    let tab_id = TabId::new();
                    self.pages.insert(tab_id, PageState::new_tab());
                    self.chrome.add_tab("New Tab", "flxtra://newtab");
                    self.chrome.switch_tab(self.chrome.tabs.len() - 1);
                    self.active_tab = Some(tab_id);
                }
                ChromeHitResult::NavButton(NavButton::Back) => {
                    info!("Back button clicked");
                }
                ChromeHitResult::NavButton(NavButton::Forward) => {
                    info!("Forward button clicked");
                }
                ChromeHitResult::NavButton(NavButton::Refresh) => {
                    info!("Refresh button clicked");
                }
                ChromeHitResult::NavButton(NavButton::Home) => {
                    // Navigate to home (new tab)
                    if let Some(tab_id) = self.active_tab {
                        if let Some(page) = self.pages.get_mut(&tab_id) {
                            page.url = "flxtra://newtab".to_string();
                            page.title = "New Tab".to_string();
                            self.chrome.set_url("flxtra://newtab");
                            self.chrome.set_title("New Tab");
                        }
                    }
                }
                ChromeHitResult::UrlBar => {
                    self.chrome.is_url_focused = !self.chrome.is_url_focused;
                    if self.chrome.is_url_focused {
                        // Copy current URL to input
                        if let Some(tab_id) = self.active_tab {
                            if let Some(page) = self.pages.get(&tab_id) {
                                self.chrome.url_input = page.url.clone();
                            }
                        }
                    }
                }
                ChromeHitResult::CloseTab(index) => {
                    self.chrome.close_tab(index);
                }
                _ => {}
            }
        }
        
        let _ = self.render();
    }

    pub fn handle_key(&mut self, key: char) {
        if self.chrome.is_url_focused {
            if key == '\r' || key == '\n' {
                // Enter pressed - would navigate
                let url = self.chrome.url_input.clone();
                self.chrome.is_url_focused = false;
                info!("Navigate to: {}", url);
                self.chrome.status_text = format!("Navigating to {}...", url);
            } else if key == '\x08' {
                // Backspace
                self.chrome.url_input.pop();
            } else if key == '\x1b' {
                // Escape - cancel
                self.chrome.is_url_focused = false;
            } else if !key.is_control() {
                self.chrome.url_input.push(key);
            }
            let _ = self.render();
        }
    }

    pub fn navigate_sync(&mut self, url: &str) {
        if let Some(tab_id) = self.active_tab {
            if let Some(page) = self.pages.get_mut(&tab_id) {
                // Handle internal URLs
                if url.starts_with("flxtra://") {
                    page.url = url.to_string();
                    page.title = match url {
                        "flxtra://newtab" => "New Tab".to_string(),
                        "flxtra://settings" => "Settings".to_string(),
                        _ => "Flxtra".to_string(),
                    };
                    page.is_loading = false;
                    page.error = None;
                    self.chrome.set_url(url);
                    self.chrome.set_title(&page.title);
                }
            }
        }
    }

    pub fn render(&mut self) -> Result<()> {
        let mut display_list = DisplayList::new();

        // Render browser chrome
        self.chrome.render(&mut display_list);

        // Render page content
        let content_area = self.chrome.content_area();
        
        if let Some(tab_id) = self.active_tab {
            if let Some(page) = self.pages.get(&tab_id) {
                if page.is_loading {
                    render_loading_page(&mut display_list, content_area, &page.url);
                } else if page.url == "flxtra://newtab" || page.url.starts_with("flxtra://") {
                    render_new_tab_page(&mut display_list, content_area);
                } else if let Some(error) = &page.error {
                    render_error_page(&mut display_list, content_area, error);
                } else {
                    // Render page display list at content area offset
                    for cmd in &page.display_list.commands {
                        display_list.push(offset_command(cmd.clone(), content_area.x, content_area.y));
                    }
                }
            }
        }

        // Render to screen
        if let Some(ref renderer) = self.renderer {
            renderer.render(&display_list, Color::new(18, 18, 20, 255));
        }

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // Create window
        let window = BrowserWindow::new(
            "Flxtra Browser",
            self.config.ui.window_width,
            self.config.ui.window_height,
        ).map_err(|e| FlxtraError::Internal(e.to_string()))?;

        // Create GDI renderer
        let mut renderer = GdiRenderer::new()
            .map_err(|e| FlxtraError::Internal(e.to_string()))?;
        
        renderer.create_render_target(
            window.hwnd,
            self.config.ui.window_width,
            self.config.ui.window_height,
        ).map_err(|e| FlxtraError::Internal(e.to_string()))?;

        self.renderer = Some(renderer);

        // Set up event callback using RefCell for interior mutability
        // We need to use raw pointers here to work around the borrow checker
        let browser_ptr = self as *mut Browser;
        window.set_event_callback(move |event| {
            unsafe {
                (*browser_ptr).handle_event(event);
            }
        });

        self.window = Some(window);

        // Initial render
        self.render()?;

        info!("Flxtra Browser running - click to interact!");

        // Run event loop
        if let Some(ref window) = self.window {
            window.run_event_loop();
        }

        Ok(())
    }
}

fn offset_command(cmd: flxtra_render::DisplayCommand, dx: f32, dy: f32) -> flxtra_render::DisplayCommand {
    use flxtra_render::DisplayCommand::*;
    match cmd {
        SolidColor { color, rect } => SolidColor {
            color,
            rect: flxtra_layout::box_model::Rect::new(rect.x + dx, rect.y + dy, rect.width, rect.height),
        },
        Text { text, x, y, color, size } => Text { text, x: x + dx, y: y + dy, color, size },
        Border { rect, color, width } => Border {
            rect: flxtra_layout::box_model::Rect::new(rect.x + dx, rect.y + dy, rect.width, rect.height),
            color,
            width,
        },
        Image { url, rect } => Image {
            url,
            rect: flxtra_layout::box_model::Rect::new(rect.x + dx, rect.y + dy, rect.width, rect.height),
        },
    }
}
