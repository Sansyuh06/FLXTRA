//! Browser Tab
use flxtra_core::{ContainerId, LoadState, Result, ResourceType, SecurityState, TabId};
use flxtra_html::{HtmlParser, DomTree};
use flxtra_css::CssParser;
use flxtra_js::JsRuntime;
use flxtra_layout::{LayoutTree, LayoutEngine};
use flxtra_net::NetworkClient;
use flxtra_render::{DisplayList, Painter};
use std::sync::Arc;
use tracing::{debug, info};

pub struct Tab {
    pub id: TabId,
    pub container_id: ContainerId,
    pub url: String,
    pub title: String,
    pub load_state: LoadState,
    pub security: SecurityState,
    display_list: Option<DisplayList>,
}

impl Tab {
    pub fn new(container_id: ContainerId) -> Self {
        Self {
            id: TabId::new(),
            container_id,
            url: String::new(),
            title: "New Tab".to_string(),
            load_state: LoadState::NotStarted,
            security: SecurityState::default(),
            display_list: None,
        }
    }

    pub async fn navigate(&mut self, url: &str, network: Arc<NetworkClient>) -> Result<()> {
        info!("Navigating to: {}", url);
        self.url = url.to_string();
        self.load_state = LoadState::Loading;

        // Handle internal URLs
        if url.starts_with("Flxtra://") {
            return self.handle_internal_url(url);
        }

        // Fetch the page
        let response = network.fetch(url, ResourceType::Document, None).await?;
        let html = String::from_utf8_lossy(&response.body).to_string();
        debug!("Received {} bytes", html.len());

        // Parse HTML
        let parser = HtmlParser::new();
        let document = parser.parse(&html);
        let dom_tree = DomTree::new(document);

        // Update title
        if let Some(title) = dom_tree.title() {
            self.title = title;
        }

        // Build layout tree
        if let Some(doc_elem) = dom_tree.document_element() {
            let mut layout_root = LayoutTree::build(&doc_elem.node);
            
            // Perform layout
            let mut engine = LayoutEngine::new(1280.0, 720.0);
            engine.layout(&mut layout_root);

            // Paint to display list
            self.display_list = Some(Painter::paint(&layout_root));
        }

        self.load_state = LoadState::Complete;
        self.security.is_secure = url.starts_with("https://");
        
        info!("Page loaded: {}", self.title);
        Ok(())
    }

    fn handle_internal_url(&mut self, url: &str) -> Result<()> {
        match url {
            "Flxtra://newtab" => {
                self.title = "New Tab".to_string();
                self.load_state = LoadState::Complete;
            }
            "Flxtra://settings" => {
                self.title = "Settings".to_string();
                self.load_state = LoadState::Complete;
            }
            "Flxtra://privacy" => {
                self.title = "Privacy Dashboard".to_string();
                self.load_state = LoadState::Complete;
            }
            _ => {
                self.title = "Page Not Found".to_string();
                self.load_state = LoadState::Failed;
            }
        }
        Ok(())
    }

    pub fn get_display_list(&self) -> Option<&DisplayList> {
        self.display_list.as_ref()
    }
}
