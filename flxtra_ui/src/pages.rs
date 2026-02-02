//! New Tab Page - Welcome page for Flxtra browser

use flxtra_css::values::Color;
use flxtra_layout::box_model::Rect;
use flxtra_render::{DisplayList, DisplayCommand};

/// Renders the new tab page
pub fn render_new_tab_page(list: &mut DisplayList, content_area: Rect) {
    let center_x = content_area.x + content_area.width / 2.0;
    let center_y = content_area.y + content_area.height / 2.0;

    // Background
    list.fill_rect(content_area, Color::new(18, 18, 20, 255));

    // Logo / Title
    list.draw_text(
        "Flxtra".to_string(),
        center_x - 80.0,
        center_y - 100.0,
        Color::new(100, 149, 237, 255), // Cornflower blue
        48.0,
    );

    // Tagline
    list.draw_text(
        "Privacy-First Browsing".to_string(),
        center_x - 100.0,
        center_y - 40.0,
        Color::new(180, 180, 180, 255),
        16.0,
    );

    // Search box placeholder
    let search_rect = Rect::new(center_x - 250.0, center_y, 500.0, 45.0);
    list.fill_rect(search_rect, Color::new(35, 35, 38, 255));
    list.draw_border(search_rect, Color::new(70, 70, 73, 255), 1.0);
    list.draw_text(
        "Search or enter URL...".to_string(),
        center_x - 230.0,
        center_y + 28.0,
        Color::new(120, 120, 120, 255),
        14.0,
    );

    // Quick links
    let links = [
        ("📰", "News"),
        ("🎬", "Videos"),
        ("📧", "Email"),
        ("📁", "Files"),
    ];

    let link_y = center_y + 100.0;
    let link_start_x = center_x - 200.0;
    
    for (i, (icon, label)) in links.iter().enumerate() {
        let x = link_start_x + (i as f32 * 100.0);
        
        // Link box
        list.fill_rect(
            Rect::new(x, link_y, 80.0, 70.0),
            Color::new(40, 40, 43, 255),
        );
        
        // Icon
        list.draw_text(
            icon.to_string(),
            x + 28.0,
            link_y + 35.0,
            Color::WHITE,
            24.0,
        );
        
        // Label
        list.draw_text(
            label.to_string(),
            x + 20.0,
            link_y + 60.0,
            Color::new(180, 180, 180, 255),
            11.0,
        );
    }

    // Privacy stats
    list.draw_text(
        "🛡 Trackers blocked: 0  |  🔒 HTTPS enforced  |  🌐 DNS-over-HTTPS".to_string(),
        center_x - 200.0,
        center_y + 200.0,
        Color::new(100, 100, 100, 255),
        12.0,
    );
}

/// Renders a loading page
pub fn render_loading_page(list: &mut DisplayList, content_area: Rect, url: &str) {
    list.fill_rect(content_area, Color::new(18, 18, 20, 255));
    
    let center_x = content_area.x + content_area.width / 2.0;
    let center_y = content_area.y + content_area.height / 2.0;

    list.draw_text(
        "Loading...".to_string(),
        center_x - 40.0,
        center_y - 20.0,
        Color::WHITE,
        18.0,
    );

    list.draw_text(
        url.to_string(),
        center_x - 150.0,
        center_y + 20.0,
        Color::new(100, 149, 237, 255),
        12.0,
    );
}

/// Renders an error page
pub fn render_error_page(list: &mut DisplayList, content_area: Rect, error: &str) {
    list.fill_rect(content_area, Color::new(18, 18, 20, 255));
    
    let center_x = content_area.x + content_area.width / 2.0;
    let center_y = content_area.y + content_area.height / 2.0;

    list.draw_text(
        "⚠ Page Load Error".to_string(),
        center_x - 80.0,
        center_y - 40.0,
        Color::new(244, 67, 54, 255),
        20.0,
    );

    list.draw_text(
        error.to_string(),
        center_x - 150.0,
        center_y + 10.0,
        Color::new(180, 180, 180, 255),
        14.0,
    );
}
