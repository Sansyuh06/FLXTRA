//! Main Browser controller
use flxtra_core::{BrowserConfig, ContainerId, NavigationRequest, TabId, Result, FlxtraError};
use flxtra_filter::FilterEngine;
use flxtra_mcp::McpClient;
use flxtra_net::NetworkClient;
use flxtra_sandbox::container::ContainerManager;
use flxtra_ui::BrowserWindow;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::tab::Tab;

pub struct Browser {
    config: BrowserConfig,
    tabs: HashMap<TabId, Tab>,
    active_tab: Option<TabId>,
    network: Arc<NetworkClient>,
    filter: Arc<FilterEngine>,
    containers: ContainerManager,
    mcp: McpClient,
    window: Option<BrowserWindow>,
}

impl Browser {
    pub async fn new(config: BrowserConfig) -> Result<Self> {
        info!("Initializing Flxtra Browser");

        // Initialize network client
        let network = Arc::new(
            NetworkClient::new(
                &config.network.doh_server,
                config.network.allow_http,
                config.network.timeout_seconds as u64,
            )
            .await?,
        );

        // Initialize filter engine
        let filter = Arc::new(FilterEngine::new());

        // Load filter lists
        let filter_lists = vec![
            "https://easylist.to/easylist/easylist.txt".to_string(),
            "https://easylist.to/easylist/easyprivacy.txt".to_string(),
        ];
        network.load_filters(&filter_lists).await?;

        // Initialize containers
        let mut containers = ContainerManager::new();
        containers.add(ContainerManager::default_container());
        containers.add(ContainerManager::banking_container());
        containers.add(ContainerManager::social_container());

        Ok(Self {
            config,
            tabs: HashMap::new(),
            active_tab: None,
            network,
            filter,
            containers,
            mcp: McpClient::new(),
            window: None,
        })
    }

    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        let tab_id = match self.active_tab {
            Some(id) => id,
            None => {
                let id = self.new_tab()?;
                self.active_tab = Some(id);
                id
            }
        };

        let tab = self.tabs.get_mut(&tab_id).ok_or(FlxtraError::NotFound("Tab".into()))?;
        tab.navigate(url, self.network.clone()).await
    }

    pub fn new_tab(&mut self) -> Result<TabId> {
        let tab = Tab::new(ContainerId::default());
        let id = tab.id;
        self.tabs.insert(id, tab);
        info!("Created new tab: {:?}", id);
        Ok(id)
    }

    pub fn close_tab(&mut self, id: TabId) {
        self.tabs.remove(&id);
        if self.active_tab == Some(id) {
            self.active_tab = self.tabs.keys().next().copied();
        }
        info!("Closed tab: {:?}", id);
    }

    pub async fn run(&mut self) -> Result<()> {
        // Create browser window
        let window = BrowserWindow::new(
            "Flxtra Browser",
            self.config.ui.window_width,
            self.config.ui.window_height,
        ).map_err(|e| FlxtraError::Internal(e.to_string()))?;

        info!("Browser window created");
        window.run_event_loop();

        Ok(())
    }

    pub fn blocked_count(&self) -> u64 {
        self.filter.blocked_count()
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
}
