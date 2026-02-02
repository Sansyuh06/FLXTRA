//! Main Browser controller with full rendering
use flxtra_core::{BrowserConfig, ContainerId, TabId, Result, AegisError};
use flxtra_css::values::Color;
use flxtra_filter::FilterEngine;
use flxtra_html::{HtmlParser, DomTree};
use flxtra_layout::{LayoutTree, LayoutEngine, LayoutBox};
use flxtra_mcp::McpClient;
use flxtra_net::NetworkClient;
use flxtra_render::{DisplayList, Painter, D2DRenderer};
use flxtra_sandbox::container::ContainerManager;
use flxtra_ui::{BrowserWindow, BrowserChrome, ChromeHitResult, NavButton, render_new_tab_page, render_loading_page};
use std::collections::HashMap;
use std::sync::Arc;
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
    renderer: Option<D2DRenderer>,
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

    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        let tab_id = match self.active_tab {
            Some(id) => id,
            None => return Ok(()),
        };

        info!("Navigating to: {}", url);
        
        // Update chrome
        self.chrome.set_url(url);
        self.chrome.set_loading(true);

        // Get or create page state
        let page = self.pages.entry(tab_id).or_insert_with(PageState::new_tab);
        page.url = url.to_string();
        page.is_loading = true;
        page.error = None;

        // Handle internal URLs
        if url.starts_with("flxtra://") {
            page.is_loading = false;
            page.title = match url {
                "flxtra://newtab" => "New Tab".to_string(),
                "flxtra://settings" => "Settings".to_string(),
                "flxtra://privacy" => "Privacy Dashboard".to_string(),
                _ => "Flxtra".to_string(),
            };
            self.chrome.set_title(&page.title);
            self.chrome.set_loading(false);
            return self.render();
        }

        // Fetch page
        match self.network.fetch(url, ResourceType::Document, None).await {
            Ok(response) => {
                let html = String::from_utf8_lossy(&response.body).to_string();
                debug!("Received {} bytes", html.len());

                // Parse HTML
                let parser = HtmlParser::new();
                let document = parser.parse(&html);
                let dom_tree = DomTree::new(document);

                // Get title
                if let Some(title) = dom_tree.title() {
                    page.title = title.clone();
                    self.chrome.set_title(&title);
                }

                // Build layout
                if let Some(doc_elem) = dom_tree.document_element() {
                    let mut layout_root = LayoutTree::build(&doc_elem.node);
                    let content = self.chrome.content_area();
                    let engine = LayoutEngine::new(content.width, content.height);
                    engine.layout(&mut layout_root);
                    
                    // Paint to display list
                    page.display_list = Painter::paint(&layout_root);
                    page.layout_root = Some(layout_root);
                }

                page.is_loading = false;
            }
            Err(e) => {
                error!("Failed to load page: {}", e);
                page.error = Some(e.to_string());
                page.is_loading = false;
            }
        }

        self.chrome.set_loading(false);
        self.chrome.update_blocked_count(self.filter.blocked_count());
        self.render()
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
                } else if page.url == "flxtra://newtab" {
                    render_new_tab_page(&mut display_list, content_area);
                } else if let Some(error) = &page.error {
                    flxtra_ui::render_error_page(&mut display_list, content_area, error);
                } else {
                    // Render page display list at content area offset
                    for cmd in &page.display_list.commands {
                        display_list.push(Self::offset_command(cmd.clone(), content_area.x, content_area.y));
                    }
                }
            }
        }

        // Render to screen
        if let Some(renderer) = &self.renderer {
            renderer.render(&display_list, Color::new(18, 18, 20, 255));
        }

        Ok(())
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

    pub fn handle_click(&mut self, x: f32, y: f32) -> Result<()> {
        if let Some(hit) = self.chrome.hit_test(x, y) {
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
                    // TODO: History navigation
                }
                ChromeHitResult::NavButton(NavButton::Forward) => {
                    // TODO: History navigation
                }
                ChromeHitResult::NavButton(NavButton::Refresh) => {
                    if let Some(page) = self.active_tab.and_then(|id| self.pages.get(&id)) {
                        let url = page.url.clone();
                        // Would need async context to navigate
                    }
                }
                ChromeHitResult::NavButton(NavButton::Home) => {
                    // Would need async context to navigate
                }
                ChromeHitResult::UrlBar => {
                    self.chrome.is_url_focused = true;
                }
                _ => {}
            }
            self.render()?;
        }
        Ok(())
    }

    pub fn handle_key(&mut self, key: char) {
        if self.chrome.is_url_focused {
            if key == '\r' {
                // Enter pressed - navigate
                let url = self.chrome.url_input.clone();
                self.chrome.is_url_focused = false;
                // Would need async context to navigate
            } else if key == '\x08' {
                // Backspace
                self.chrome.url_input.pop();
            } else {
                self.chrome.url_input.push(key);
            }
            let _ = self.render();
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Create window
        let window = BrowserWindow::new(
            "Flxtra Browser",
            self.config.ui.window_width,
            self.config.ui.window_height,
        ).map_err(|e| flxtra_core::AegisError::Internal(e.to_string()))?;

        // Create D2D renderer
        let mut renderer = D2DRenderer::new()
            .map_err(|e| flxtra_core::AegisError::Internal(e.to_string()))?;
        
        renderer.create_render_target(
            window.hwnd,
            self.config.ui.window_width,
            self.config.ui.window_height,
        ).map_err(|e| flxtra_core::AegisError::Internal(e.to_string()))?;

        self.renderer = Some(renderer);
        self.window = Some(window);

        // Initial render
        self.render()?;

        info!("Flxtra Browser running");

        // Run event loop
        if let Some(ref window) = self.window {
            window.run_event_loop();
        }

        Ok(())
    }
}
