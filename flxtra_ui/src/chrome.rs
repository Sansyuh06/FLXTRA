//! Browser Chrome - UI components for the browser shell

use flxtra_css::values::Color;
use flxtra_layout::box_model::Rect;
use flxtra_render::{DisplayList, DisplayCommand};

/// Browser chrome dimensions
pub const TOOLBAR_HEIGHT: f32 = 40.0;
pub const TAB_BAR_HEIGHT: f32 = 35.0;
pub const STATUS_BAR_HEIGHT: f32 = 24.0;
pub const URL_BAR_HEIGHT: f32 = 28.0;

/// Tab information
#[derive(Debug, Clone)]
pub struct TabInfo {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    pub is_loading: bool,
    pub is_secure: bool,
}

/// Navigation button
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavButton {
    Back,
    Forward,
    Refresh,
    Home,
    Stop,
}

/// Browser chrome state
pub struct BrowserChrome {
    pub tabs: Vec<TabInfo>,
    pub active_tab_id: Option<u64>,
    pub url_input: String,
    pub is_url_focused: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub blocked_count: u64,
    pub status_text: String,
    viewport_width: f32,
    viewport_height: f32,
}

impl BrowserChrome {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            tabs: vec![TabInfo {
                id: 1,
                title: "New Tab".to_string(),
                url: "flxtra://newtab".to_string(),
                is_active: true,
                is_loading: false,
                is_secure: true,
            }],
            active_tab_id: Some(1),
            url_input: String::new(),
            is_url_focused: false,
            can_go_back: false,
            can_go_forward: false,
            blocked_count: 0,
            status_text: String::new(),
            viewport_width: width,
            viewport_height: height,
        }
    }

    pub fn content_area(&self) -> Rect {
        Rect {
            x: 0.0,
            y: TAB_BAR_HEIGHT + TOOLBAR_HEIGHT,
            width: self.viewport_width,
            height: self.viewport_height - TAB_BAR_HEIGHT - TOOLBAR_HEIGHT - STATUS_BAR_HEIGHT,
        }
    }

    pub fn render(&self, list: &mut DisplayList) {
        self.render_tab_bar(list);
        self.render_toolbar(list);
        self.render_status_bar(list);
    }

    fn render_tab_bar(&self, list: &mut DisplayList) {
        // Tab bar background
        list.fill_rect(
            Rect::new(0.0, 0.0, self.viewport_width, TAB_BAR_HEIGHT),
            Color::new(45, 45, 48, 255), // Dark gray
        );

        // Render tabs
        let mut x = 5.0;
        for tab in &self.tabs {
            let tab_width = 180.0;
            let bg_color = if tab.is_active {
                Color::new(60, 60, 63, 255)
            } else {
                Color::new(50, 50, 53, 255)
            };

            // Tab background
            list.fill_rect(Rect::new(x, 4.0, tab_width, TAB_BAR_HEIGHT - 4.0), bg_color);

            // Tab title
            let title = if tab.title.len() > 20 {
                format!("{}...", &tab.title[..17])
            } else {
                tab.title.clone()
            };
            list.draw_text(title, x + 8.0, 22.0, Color::WHITE, 12.0);

            // Loading indicator
            if tab.is_loading {
                list.draw_text("⟳".to_string(), x + tab_width - 20.0, 22.0, Color::new(100, 149, 237, 255), 12.0);
            }

            x += tab_width + 2.0;
        }

        // New tab button
        list.fill_rect(Rect::new(x, 4.0, 30.0, TAB_BAR_HEIGHT - 4.0), Color::new(55, 55, 58, 255));
        list.draw_text("+".to_string(), x + 10.0, 22.0, Color::WHITE, 14.0);
    }

    fn render_toolbar(&self, list: &mut DisplayList) {
        let y = TAB_BAR_HEIGHT;

        // Toolbar background
        list.fill_rect(
            Rect::new(0.0, y, self.viewport_width, TOOLBAR_HEIGHT),
            Color::new(35, 35, 38, 255),
        );

        // Navigation buttons
        let btn_y = y + 6.0;
        let btn_size = 28.0;
        let mut x = 8.0;

        // Back button
        let back_color = if self.can_go_back { Color::WHITE } else { Color::new(100, 100, 100, 255) };
        list.draw_text("◀".to_string(), x + 8.0, btn_y + 18.0, back_color, 14.0);
        x += btn_size + 4.0;

        // Forward button  
        let fwd_color = if self.can_go_forward { Color::WHITE } else { Color::new(100, 100, 100, 255) };
        list.draw_text("▶".to_string(), x + 8.0, btn_y + 18.0, fwd_color, 14.0);
        x += btn_size + 4.0;

        // Refresh button
        list.draw_text("↻".to_string(), x + 6.0, btn_y + 18.0, Color::WHITE, 14.0);
        x += btn_size + 4.0;

        // Home button
        list.draw_text("⌂".to_string(), x + 6.0, btn_y + 18.0, Color::WHITE, 14.0);
        x += btn_size + 12.0;

        // URL bar
        let url_bar_width = self.viewport_width - x - 100.0;
        let url_bar_rect = Rect::new(x, btn_y, url_bar_width, URL_BAR_HEIGHT);
        
        // URL bar background
        list.fill_rect(url_bar_rect, Color::new(25, 25, 28, 255));
        list.draw_border(url_bar_rect, Color::new(70, 70, 73, 255), 1.0);

        // Security indicator
        if let Some(tab) = self.tabs.iter().find(|t| t.is_active) {
            let lock_color = if tab.is_secure {
                Color::new(76, 175, 80, 255) // Green
            } else {
                Color::new(244, 67, 54, 255) // Red
            };
            list.draw_text("🔒".to_string(), x + 6.0, btn_y + 18.0, lock_color, 12.0);
        }

        // URL text
        let url_text = if self.is_url_focused {
            &self.url_input
        } else if let Some(tab) = self.tabs.iter().find(|t| t.is_active) {
            &tab.url
        } else {
            ""
        };
        list.draw_text(url_text.to_string(), x + 24.0, btn_y + 18.0, Color::WHITE, 13.0);

        // Blocked count
        if self.blocked_count > 0 {
            let blocked_x = self.viewport_width - 90.0;
            list.fill_rect(Rect::new(blocked_x, btn_y, 80.0, URL_BAR_HEIGHT), Color::new(183, 28, 28, 255));
            list.draw_text(
                format!("🛡 {}", self.blocked_count),
                blocked_x + 8.0,
                btn_y + 18.0,
                Color::WHITE,
                12.0,
            );
        }
    }

    fn render_status_bar(&self, list: &mut DisplayList) {
        let y = self.viewport_height - STATUS_BAR_HEIGHT;

        // Status bar background
        list.fill_rect(
            Rect::new(0.0, y, self.viewport_width, STATUS_BAR_HEIGHT),
            Color::new(30, 30, 33, 255),
        );

        // Status text
        if !self.status_text.is_empty() {
            list.draw_text(self.status_text.clone(), 8.0, y + 16.0, Color::new(180, 180, 180, 255), 11.0);
        }
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<ChromeHitResult> {
        // Tab bar
        if y < TAB_BAR_HEIGHT {
            let mut tab_x = 5.0;
            for (i, _) in self.tabs.iter().enumerate() {
                if x >= tab_x && x < tab_x + 180.0 {
                    return Some(ChromeHitResult::Tab(i));
                }
                tab_x += 182.0;
            }
            if x >= tab_x && x < tab_x + 30.0 {
                return Some(ChromeHitResult::NewTab);
            }
        }

        // Toolbar
        if y >= TAB_BAR_HEIGHT && y < TAB_BAR_HEIGHT + TOOLBAR_HEIGHT {
            let btn_y = TAB_BAR_HEIGHT + 6.0;
            let mut x_pos = 8.0;
            
            if x >= x_pos && x < x_pos + 28.0 {
                return Some(ChromeHitResult::NavButton(NavButton::Back));
            }
            x_pos += 32.0;
            
            if x >= x_pos && x < x_pos + 28.0 {
                return Some(ChromeHitResult::NavButton(NavButton::Forward));
            }
            x_pos += 32.0;
            
            if x >= x_pos && x < x_pos + 28.0 {
                return Some(ChromeHitResult::NavButton(NavButton::Refresh));
            }
            x_pos += 32.0;
            
            if x >= x_pos && x < x_pos + 28.0 {
                return Some(ChromeHitResult::NavButton(NavButton::Home));
            }
            x_pos += 40.0;

            let url_bar_width = self.viewport_width - x_pos - 100.0;
            if x >= x_pos && x < x_pos + url_bar_width {
                return Some(ChromeHitResult::UrlBar);
            }
        }

        None
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    pub fn set_url(&mut self, url: &str) {
        self.url_input = url.to_string();
    }

    pub fn add_tab(&mut self, title: &str, url: &str) -> u64 {
        let id = self.tabs.len() as u64 + 1;
        self.tabs.push(TabInfo {
            id,
            title: title.to_string(),
            url: url.to_string(),
            is_active: false,
            is_loading: false,
            is_secure: url.starts_with("https://") || url.starts_with("flxtra://"),
        });
        id
    }

    pub fn switch_tab(&mut self, index: usize) {
        for (i, tab) in self.tabs.iter_mut().enumerate() {
            tab.is_active = i == index;
            if i == index {
                self.active_tab_id = Some(tab.id);
            }
        }
    }

    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() > 1 && index < self.tabs.len() {
            let was_active = self.tabs[index].is_active;
            self.tabs.remove(index);
            
            if was_active && !self.tabs.is_empty() {
                let new_active = index.min(self.tabs.len() - 1);
                self.tabs[new_active].is_active = true;
                self.active_tab_id = Some(self.tabs[new_active].id);
            }
        }
    }

    pub fn set_loading(&mut self, loading: bool) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.is_active) {
            tab.is_loading = loading;
        }
    }

    pub fn set_title(&mut self, title: &str) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.is_active) {
            tab.title = title.to_string();
        }
    }

    pub fn update_blocked_count(&mut self, count: u64) {
        self.blocked_count = count;
    }
}

/// Hit test result for chrome UI
#[derive(Debug, Clone)]
pub enum ChromeHitResult {
    Tab(usize),
    NewTab,
    CloseTab(usize),
    NavButton(NavButton),
    UrlBar,
    StatusBar,
}
