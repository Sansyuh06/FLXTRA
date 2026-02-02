//! Aegis Browser - Main executable

use aegis_browser::Browser;
use aegis_core::BrowserConfig;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("aegis=info".parse()?))
        .init();

    info!("Starting Aegis Browser v{}", env!("CARGO_PKG_VERSION"));

    // Load or create configuration
    let config = BrowserConfig::default();
    info!("Privacy level: {:?}", config.privacy_level);
    info!("Performance mode: {:?}", config.performance_mode);

    // Create and run browser
    let mut browser = Browser::new(config).await?;
    
    // Navigate to homepage
    browser.navigate("aegis://newtab").await?;

    // Run the browser
    browser.run().await?;

    info!("Aegis Browser shutdown complete");
    Ok(())
}
