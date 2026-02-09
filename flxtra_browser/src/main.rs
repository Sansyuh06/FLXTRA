//! Flxtra Browser - Comet Clone (Dual-WebView + Tab Virtualization)
//!
//! Two webviews: 
//! 1. Sidebar (UI) - Shared Environment
//! 2. Content Tabs - ISOLATED Environments (Unique UserDataFolder per tab)

use tracing::{info, error, debug};
use tracing_subscriber::{fmt, EnvFilter};
use webview2::Controller;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, RECT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

// Helper to convert string to Windows wide string
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}



mod agent;
use agent::{AgentPlan, DOMItem, call_ai, call_agent_planner};

// Topbar height (horizontal layout)
const TOPBAR_HEIGHT: i32 = 90;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TabInfo {
    id: Uuid,
    title: String,
    url: String,
    favicon: Option<String>,
    active: bool,
}

impl TabInfo {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "New Tab".to_string(),
            url: "".to_string(),
            favicon: Some("✨".to_string()),
            active: true,
        }
    }
}

struct BrowserState {
    hwnd: Option<HWND>, // Store HWND for resizing
    sidebar_controller: Option<Rc<Controller>>,
    ai_sidebar_controller: Option<Rc<Controller>>,  // Marceline AI panel
    ai_sidebar_open: bool,                           // Toggle state
    // Map Tab ID -> Controller. Each controller has its own Environment (Profile)
    content_controllers: HashMap<Uuid, Rc<Controller>>,
    tabs: Vec<TabInfo>,
    active_tab_id: Uuid,
    pending_plan: Option<AgentPlan>,
}

impl BrowserState {
    fn new() -> Self {
        let initial_tab = TabInfo::new();
        
        // state.load_session(); // Disabled temporarily while refactoring structure
        Self {
            hwnd: None,
            sidebar_controller: None,
            ai_sidebar_controller: None,
            ai_sidebar_open: false,
            content_controllers: HashMap::new(),
            tabs: vec![initial_tab.clone()],
            active_tab_id: initial_tab.id,
            pending_plan: None,
        }
    }

    fn sync_sidebar(&self) {
        if let Some(ctrl) = &self.sidebar_controller {
            if let Ok(wv) = ctrl.get_webview() {
                let json = serde_json::json!({
                    "type": "update-tabs",
                    "tabs": self.tabs
                });
                let _ = wv.post_web_message_as_json(&json.to_string());
            }
        }
    }
    
    // Resize the active tab to fill content area, hide others
    fn layout_content(&self) {
        let hwnd = match self.hwnd { Some(h) => h, None => return };
        
        let mut rect = RECT::default();
        unsafe { GetClientRect(hwnd, &mut rect); }
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        
        // Calculate content width (shrink if AI sidebar is open)
        let ai_sidebar_width = if self.ai_sidebar_open { 320 } else { 0 };
        let content_right = width - ai_sidebar_width;
        
        let visible_rect = winapi::shared::windef::RECT {
            left: 0, top: TOPBAR_HEIGHT,
            right: content_right, bottom: height
        };
        
        // Position AI sidebar on the right
        if let Some(ai_ctrl) = &self.ai_sidebar_controller {
            if self.ai_sidebar_open {
                let ai_rect = winapi::shared::windef::RECT {
                    left: content_right, top: TOPBAR_HEIGHT,
                    right: width, bottom: height
                };
                let _ = ai_ctrl.put_bounds(ai_rect);
                let _ = ai_ctrl.put_is_visible(true);
            } else {
                let _ = ai_ctrl.put_is_visible(false);
            }
        }
        
        // Hide others (move offscreen/zero size)
        let _hidden_rect = winapi::shared::windef::RECT { left: 0, top: 0, right: 0, bottom: 0 };

        for (id, ctrl) in &self.content_controllers {
            if *id == self.active_tab_id {
                let _ = ctrl.put_bounds(visible_rect);
                let _ = ctrl.put_is_visible(true);
            } else {
                let _ = ctrl.put_bounds(visible_rect);
                let _ = ctrl.put_is_visible(false);
            }
        }
    }
}

thread_local! {
    static STATE: RefCell<BrowserState> = RefCell::new(BrowserState::new());
}

fn main() -> anyhow::Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env().add_directive("Flxtra=info".parse()?)).init();
    info!("Starting Flextra Browser...");

    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_base = to_wstring("FlxtraCometClass");
        
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_base.as_ptr()),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0 as _),
            ..Default::default()
        };
        
        RegisterClassExW(&wc);
        
        let hwnd_res = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_base.as_ptr()),
            PCWSTR(to_wstring("Flextra").as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT, CW_USEDEFAULT,
            1400, 900,
            HWND::default(), None, instance, None,
        );

        let hwnd = hwnd_res?;
        if hwnd.0.is_null() {
            error!("Failed to create window");
            return Ok(());
        }
        
        STATE.with(|s| s.borrow_mut().hwnd = Some(hwnd));

        // Initialize UI Shell
        init_sidebar(hwnd)?;
        
        // Initialize First Tab (Isolated)
        let first_tab_id = STATE.with(|s| s.borrow().active_tab_id);
        create_isolated_tab(hwnd, first_tab_id)?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

fn init_sidebar(hwnd: HWND) -> anyhow::Result<()> {
    webview2::Environment::builder().build(move |env| {
        let env = env.map_err(|e| { error!("Env error: {:?}", e); e })?;
        let winapi_hwnd = hwnd.0 as *mut winapi::shared::windef::HWND__;
        
        env.create_controller(winapi_hwnd, move |ctrl| {
            let ctrl = ctrl.map_err(|e| { error!("Sidebar ctrl error: {:?}", e); e })?;
            let webview = ctrl.get_webview()?;
            
            // Layout
            let mut rect = RECT::default();
            unsafe { GetClientRect(hwnd, &mut rect); }
            let side_rect = winapi::shared::windef::RECT {
                left: 0, top: 0,
                right: rect.right - rect.left,
                bottom: TOPBAR_HEIGHT,
            };
            ctrl.put_bounds(side_rect)?;
            
            // Load Sidebar (check multiple paths for dev/release compatibility)
            let cwd = std::env::current_dir().unwrap_or_default();
            let sidebar_path = if cwd.join("src/sidebar.html").exists() {
                cwd.join("src/sidebar.html")
            } else if cwd.join("flxtra_browser/src/sidebar.html").exists() {
                cwd.join("flxtra_browser/src/sidebar.html")
            } else {
                cwd.join("sidebar.html")
            };
            let sidebar_url = format!("file:///{}", sidebar_path.to_str().unwrap_or("").replace("\\", "/"));
            webview.navigate(&sidebar_url)?;
            
            // Message Handler
            webview.add_web_message_received(move |_, args| {
                let msg = args.get_web_message_as_json()?;
                let val: serde_json::Value = serde_json::from_str(&msg).unwrap_or_default();
                
                if let Some(cmd) = val["command"].as_str() {
                    match cmd {
                        "new-tab" => {
                            // Create new isolated tab
                            let new_tab = TabInfo::new();
                            let new_id = new_tab.id;
                            
                            STATE.with(|s| {
                                let mut state = s.borrow_mut();
                                state.tabs.push(new_tab);
                                for t in &mut state.tabs { t.active = false; }
                                if let Some(last) = state.tabs.last_mut() {
                                    last.active = true;
                                }
                                state.active_tab_id = new_id;
                                state.sync_sidebar();
                            });
                            
                            // Async create environment
                            if let Some(h) = STATE.with(|s| s.borrow().hwnd) {
                                let _ = create_isolated_tab(h, new_id);
                            }
                        },
                        "switch-tab" => {
                            // index provided
                             if let Some(idx) = val["data"].as_u64() {
                                 let idx = idx as usize;
                                 STATE.with(|s| {
                                     let mut state = s.borrow_mut();
                                     if idx < state.tabs.len() {
                                         let id = state.tabs[idx].id;
                                         state.active_tab_id = id;
                                         for (i, t) in state.tabs.iter_mut().enumerate() {
                                             t.active = i == idx;
                                         }
                                         state.sync_sidebar();
                                         state.layout_content();
                                     }
                                 });
                             }
                        },
                        "close-tab" => {
                            if let Some(idx) = val["data"].as_u64() {
                                let idx = idx as usize;
                                STATE.with(|s| {
                                    let mut state = s.borrow_mut();
                                    if idx < state.tabs.len() && state.tabs.len() > 1 {
                                        let tab_id = state.tabs[idx].id;
                                        
                                        // Remove tab
                                        state.tabs.remove(idx);
                                        state.content_controllers.remove(&tab_id);
                                        
                                        // Switch to another tab
                                        let new_idx = if idx >= state.tabs.len() { state.tabs.len() - 1 } else { idx };
                                        state.active_tab_id = state.tabs[new_idx].id;
                                        for (i, t) in state.tabs.iter_mut().enumerate() {
                                            t.active = i == new_idx;
                                        }
                                        
                                        state.sync_sidebar();
                                        state.layout_content();
                                    }
                                });
                            }
                        },
                        "toggle-ai" => {
                            // Toggle Marceline AI sidebar
                            let hwnd_opt = STATE.with(|s| s.borrow().hwnd);
                            let has_ai_sidebar = STATE.with(|s| s.borrow().ai_sidebar_controller.is_some());
                            
                            if has_ai_sidebar {
                                // Just toggle visibility
                                STATE.with(|s| {
                                    let mut state = s.borrow_mut();
                                    state.ai_sidebar_open = !state.ai_sidebar_open;
                                    state.layout_content();
                                });
                            } else if let Some(hwnd) = hwnd_opt {
                                // Create AI sidebar WebView
                                let _ = create_ai_sidebar(hwnd);
                                STATE.with(|s| {
                                    let mut state = s.borrow_mut();
                                    state.ai_sidebar_open = true;
                                });
                            }
                        },
                        "ai-scan" => {
                            // Trigger analysis on active tab
                             let active_id = STATE.with(|s| s.borrow().active_tab_id);
                             STATE.with(|s| {
                                 if let Some(ctrl) = s.borrow().content_controllers.get(&active_id) {
                                     if let Ok(wv) = ctrl.get_webview() {
                                         // Inject script to get text content
                                         let sidebar_ctrl = s.borrow().sidebar_controller.clone(); // Clone for closure
                                         
                                         wv.execute_script("document.body.innerText", move |text_json| {
                                             let text: String = serde_json::from_str(&text_json).unwrap_or_default();
                                             
                                             // Simple "AI" Analysis in Rust
                                             let word_count = text.split_whitespace().count();
                                             let read_time = (word_count as f64 / 200.0).ceil() as u64; // 200 wpm
                                                 let summary = if text.len() > 100 { 
                                                     format!("{}...", &text[0..100].replace('\n', " ")) 
                                                 } else { 
                                                     "Not enough content to analyze.".to_string() 
                                                 };

                                                 // Send back to Sidebar
                                                 if let Some(sb_ctrl) = &sidebar_ctrl {
                                                     if let Ok(sb_wv) = sb_ctrl.get_webview() {
                                                         let response = serde_json::json!({
                                                             "type": "ai-analysis",
                                                             "data": {
                                                                 "words": word_count,
                                                                 "time": read_time,
                                                                 "preview": summary,
                                                                 "privacy_score": 98
                                                             }
                                                         });
                                                         let _ = sb_wv.post_web_message_as_json(&response.to_string());
                                                     }
                                                 }
                                             
                                             Ok(())
                                         }).unwrap_or_else(|e| error!("Scan error: {:?}", e));
                                     }
                                 }
                             });
                        },
                        "ai-summarize" | "ai-explain" | "ai-keypoints" | "ai-ask" => {
                            let action = if cmd == "ai-ask" { "ask" } else { &cmd[3..] };
                            let question = if cmd == "ai-ask" {
                                val["data"].as_str().unwrap_or("").to_string()
                            } else {
                                String::new()
                            };

                            let active_id = STATE.with(|s| s.borrow().active_tab_id);
                            STATE.with(|s| {
                                if let Some(ctrl) = s.borrow().content_controllers.get(&active_id) {
                                    if let Ok(wv) = ctrl.get_webview() {
                                        let sidebar_ctrl = s.borrow().sidebar_controller.clone();
                                        let action_clone = action.to_string();
                                        
                                        wv.execute_script("document.body.innerText", move |text_json| {
                                            let text: String = serde_json::from_str(&text_json).unwrap_or_default();
                                            let truncated = if text.len() > 4000 { &text[..4000] } else { &text };
                                            
                                            // Call AI service
                                            let prompt = if action_clone == "ask" { &question } else { truncated };
                                            let context = if action_clone == "ask" { truncated } else { "" };
                                            
                                            let result = call_ai(prompt, &action_clone, context);
                                            
                                            if let Some(sb_ctrl) = &sidebar_ctrl {
                                                if let Ok(sb_wv) = sb_ctrl.get_webview() {
                                                    let response = serde_json::json!({
                                                        "type": "ai-result",
                                                        "action": action_clone,
                                                        "data": result
                                                    });
                                                    let _ = sb_wv.post_web_message_as_json(&response.to_string());
                                                }
                                            }
                                            Ok(())
                                        }).unwrap_or_else(|e| error!("Script exec error: {:?}", e));
                                    }
                                }
                            });
                        },
                        "agent-start" => {
                            let goal = val["data"].as_str().unwrap_or("").to_string();
                            let active_id = STATE.with(|s| s.borrow().active_tab_id);
                            
                            STATE.with(|s| {
                                if let Some(ctrl) = s.borrow().content_controllers.get(&active_id) {
                                    if let Ok(wv) = ctrl.get_webview() {
                                        let sidebar_ctrl = s.borrow().sidebar_controller.clone();
                                        
                                        // Load scanner script
                                        let script = std::fs::read_to_string("flxtra_browser/src/agent_scanner.js")
                                            .unwrap_or_else(|_| "[]".to_string());
                                            
                                        wv.execute_script(&script, move |dom_json| {
                                            let dom: Vec<DOMItem> = serde_json::from_str(&dom_json).unwrap_or_default();
                                            
                                            // Call Planner
                                            if let Some(plan) = call_agent_planner(&goal, &dom) {
                                                // Store plan
                                                STATE.with(|s| s.borrow_mut().pending_plan = Some(plan.clone()));
                                                
                                                // Notify Sidebar
                                                if let Some(sb_ctrl) = &sidebar_ctrl {
                                                    if let Ok(sb_wv) = sb_ctrl.get_webview() {
                                                        let response = serde_json::json!({
                                                            "type": "agent-plan",
                                                            "plan": plan
                                                        });
                                                        let _ = sb_wv.post_web_message_as_json(&response.to_string());
                                                     }
                                                 }
                                             }
                                             Ok(())
                                         }).unwrap_or_else(|e| error!("Agent start error: {:?}", e));
                                     }
                                 }
                             });
                        },
                        "privacy-stats" => {
                             // Open Privacy Dashboard
                             let new_tab = TabInfo {
                                 id: Uuid::new_v4(),
                                 title: "Privacy Dashboard".to_string(),
                                 url: "flxtra://privacy".to_string(),
                                 favicon: Some("🛡️".to_string()),
                                 active: true,
                             };
                             let new_id = new_tab.id;
                             
                             STATE.with(|s| {
                                 let mut state = s.borrow_mut();
                                 state.tabs.push(new_tab);
                                 for t in &mut state.tabs { t.active = false; }
                                 if let Some(last) = state.tabs.last_mut() {
                                     last.active = true;
                                 }
                                 state.active_tab_id = new_id;
                                 state.sync_sidebar();
                             });
                             
                             if let Some(h) = STATE.with(|s| s.borrow().hwnd) {
                                 let _ = create_isolated_tab(h, new_id);
                             }
                        },
                        "agent-confirm" => {
                            let active_id = STATE.with(|s| s.borrow().active_tab_id);
                            let plan = STATE.with(|s| s.borrow().pending_plan.clone());
                            
                            if let Some(p) = plan {
                                STATE.with(|s| {
                                    if let Some(ctrl) = s.borrow().content_controllers.get(&active_id) {
                                        if let Ok(wv) = ctrl.get_webview() {
                                            let script = match p.action.as_str() {
                                                "click" => format!(
                                                    "document.querySelector('[data-flxtra-id=\"{}\"]').click();", 
                                                    p.target
                                                ),
                                                "type" => {
                                                    let safe_value = serde_json::to_string(&p.value.unwrap_or_default()).unwrap_or_else(|_| "\"\"".to_string());
                                                    format!(
                                                        "let el = document.querySelector('[data-flxtra-id=\"{}\"]'); if(el) {{ el.value = {}; el.dispatchEvent(new Event('input', {{ bubbles: true }})); }}", 
                                                        p.target, safe_value
                                                    )
                                                },
                                                "scroll" => "window.scrollBy(0, 500);".to_string(),
                                                _ => "".to_string()
                                            };
                                            
                                            if !script.is_empty() {
                                                let _ = wv.execute_script(&script, |_| Ok(()));
                                            }
                                        }
                                    }
                                });
                            }
                        },
                        "navigate" => {
                            if let Some(url) = val["data"].as_str() {
                                let active_id = STATE.with(|s| s.borrow().active_tab_id);
                                STATE.with(|s| {
                                    if let Some(ctrl) = s.borrow().content_controllers.get(&active_id) {
                                        if let Ok(wv) = ctrl.get_webview() {
                                            let final_url = if url.contains('.') && !url.contains(' ') {
                                                if url.starts_with("http") { url.to_string() } else { format!("https://{}", url) }
                                            } else {
                                                format!("https://duckduckgo.com/?q={}", url)
                                            };
                                            let _ = wv.navigate(&final_url);
                                        }
                                    }
                                });
                            }
                        },
                        _ => debug!("Cmd: {}", cmd)
                    }
                }
                Ok(())
            })?;

            STATE.with(|s| s.borrow_mut().sidebar_controller = Some(Rc::new(ctrl)));
            Ok(())
        })?;
        Ok(())
    })
    .map_err(|e| anyhow::anyhow!("Sidebar Init Error: {:?}", e))?;
    Ok(())
}

fn create_ai_sidebar(hwnd: HWND) -> anyhow::Result<()> {
    webview2::Environment::builder().build(move |env| {
        let env = env.map_err(|e| { error!("AI Env error: {:?}", e); e })?;
        let winapi_hwnd = hwnd.0 as *mut winapi::shared::windef::HWND__;
        
        env.create_controller(winapi_hwnd, move |ctrl| {
            let ctrl = ctrl.map_err(|e| { error!("AI ctrl error: {:?}", e); e })?;
            let webview = ctrl.get_webview()?;
            
            // Initial layout (will be properly set by layout_content)
            let mut rect = RECT::default();
            unsafe { GetClientRect(hwnd, &mut rect); }
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            let ai_rect = winapi::shared::windef::RECT {
                left: width - 320, top: TOPBAR_HEIGHT,
                right: width, bottom: height
            };
            ctrl.put_bounds(ai_rect)?;
            
            // Load AI Panel (check multiple paths)
            let cwd = std::env::current_dir().unwrap_or_default();
            let ai_path = if cwd.join("src/ai_panel.html").exists() {
                cwd.join("src/ai_panel.html")
            } else if cwd.join("flxtra_browser/src/ai_panel.html").exists() {
                cwd.join("flxtra_browser/src/ai_panel.html")
            } else {
                cwd.join("ai_panel.html")
            };
            let ai_url = format!("file:///{}", ai_path.to_str().unwrap_or("").replace("\\", "/"));
            webview.navigate(&ai_url)?;
            
            // Handle messages from AI panel
            webview.add_web_message_received(move |_, args| {
                let msg = args.get_web_message_as_json()?;
                let val: serde_json::Value = serde_json::from_str(&msg).unwrap_or_default();
                
                if let Some(cmd) = val["command"].as_str() {
                    match cmd {
                        "toggle-ai" => {
                            // Close sidebar
                            STATE.with(|s| {
                                let mut state = s.borrow_mut();
                                state.ai_sidebar_open = false;
                                state.layout_content();
                            });
                        },
                        "ai-action" | "ai-question" => {
                            // Forward to main handler
                            let action = val["data"].as_str().unwrap_or("").to_string();
                            let is_question = cmd == "ai-question";
                            
                            // Get active tab content
                            let active_id = STATE.with(|s| s.borrow().active_tab_id);
                            let ai_sidebar_ctrl = STATE.with(|s| s.borrow().ai_sidebar_controller.clone());
                            
                            STATE.with(|s| {
                                if let Some(ctrl) = s.borrow().content_controllers.get(&active_id) {
                                    if let Ok(wv) = ctrl.get_webview() {
                                        // Capture context for closure
                                        let _ai_ctrl_clone = ai_sidebar_ctrl.clone();
                                        let action_clone = action.clone();
                                        
                                        // Execute script to get text and URL - try multiple methods
                                        let _ = wv.execute_script("(function(){var t=document.body.innerText||document.body.textContent||'';return JSON.stringify({text:t.slice(0,15000),url:location.href});})()", move |json_str| {
                                            info!("Raw script result length: {}", json_str.len());
                                            
                                            // WebView2 returns the result as a JSON-encoded string
                                            // So we need to first deserialize the outer string, then parse the inner JSON
                                            let inner_json: String = serde_json::from_str(&json_str).unwrap_or_else(|_| json_str.clone());
                                            
                                            let data: serde_json::Value = serde_json::from_str(&inner_json).unwrap_or_default();
                                            let text = data["text"].as_str().unwrap_or("").to_string();
                                            let url = data["url"].as_str().unwrap_or("").to_string();
                                            
                                            info!("Extracted {} chars from {}", text.len(), url);
                                            if text.len() > 0 {
                                                info!("Content preview: {}", text.chars().take(200).collect::<String>());
                                            }
                                            
                                            // Spawn thread to avoid blocking UI
                                            let hwnd_raw = STATE.with(|s| s.borrow().hwnd).map(|h| h.0 as usize).unwrap_or(0);
                                            if hwnd_raw != 0 {
                                                std::thread::spawn(move || {
                                                    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
                                                    
                                                    // Check if we actually got content
                                                    let response_text = if text.trim().len() < 50 {
                                                        "❌ **Could not extract page content.**\n\nThis might happen if:\n- The page is still loading\n- The page uses heavy JavaScript\n- The content is in an iframe\n\nTry refreshing the page and clicking again.".to_string()
                                                    } else if is_question {
                                                        call_ai(&action_clone, "ask", &text.chars().take(8000).collect::<String>())
                                                    } else {
                                                        match action_clone.as_str() {
                                                            "summarize" => call_ai(&text.chars().take(8000).collect::<String>(), "summarize", ""),
                                                            "explain" => call_ai(&text.chars().take(8000).collect::<String>(), "explain", ""),
                                                            "keypoints" => call_ai(&text.chars().take(8000).collect::<String>(), "keypoints", ""),
                                                            "privacy" => {
                                                                // Heuristic Privacy Check
                                                                let trackers = vec![
                                                                    ("google-analytics.com", "Google Analytics"),
                                                                    ("googletagmanager.com", "Google Tag Manager"),
                                                                    ("facebook.net", "Meta Pixel"),
                                                                    ("doubleclick.net", "Google Ads"),
                                                                    ("adservice.google.com", "Google AdServices"),
                                                                    ("clarity.ms", "Microsoft Clarity"),
                                                                    ("hotjar.com", "Hotjar"),
                                                                    ("criteo.com", "Criteo"),
                                                                    ("amazon-adsystem.com", "Amazon Ads"),
                                                                    ("bing.com/bat.js", "Bing Ads")
                                                                ];
                                                                
                                                                let found_trackers: Vec<&str> = trackers.iter()
                                                                    .filter(|(domain, _)| text.contains(domain) || text.contains(&domain.replace(".", "\\.")))
                                                                    .map(|(_, name)| *name)
                                                                    .collect();
                                                                    
                                                                let tracker_msg = if found_trackers.is_empty() {
                                                                     "✅ **No obvious trackers found.**".to_string()
                                                                } else {
                                                                     format!("⚠️ **{} Potential Trackers Detected:**\n- {}", found_trackers.len(), found_trackers.join("\n- "))
                                                                };

                                                                // Call Ollama for Deep Analysis
                                                                let analysis = call_ai(&text.chars().take(6000).collect::<String>(), "analyze", "");
                                                                
                                                                format!("🔒 **Privacy Report**\n\n{}\n\n---\n\n**AI Analysis:**\n{}", tracker_msg, analysis)
                                                            },
                                                            _ => "I'm not sure how to help with that yet.".to_string()
                                                        }
                                                    };
                                                    
                                                    // Send result back to UI thread
                                                    let boxed = Box::new(response_text);
                                                    let ptr = Box::into_raw(boxed);
                                                    unsafe { PostMessageW(hwnd, WM_APP + 1, WPARAM(ptr as usize), LPARAM(0)); }
                                                });
                                            }
                                            Ok(())
                                        });
                                    }
                                }
                            });
                        },
                        "agent-goal" => {
                            // Agent Mode: Scan DOM, call planner, return plan
                            let goal = val["data"].as_str().unwrap_or("").to_string();
                            let active_id = STATE.with(|s| s.borrow().active_tab_id);
                            
                            STATE.with(|s| {
                                if let Some(ctrl) = s.borrow().content_controllers.get(&active_id) {
                                    if let Ok(wv) = ctrl.get_webview() {
                                        // Load scanner script
                                        let script = std::fs::read_to_string("flxtra_browser/src/agent_scanner.js")
                                            .unwrap_or_else(|_| "JSON.stringify([])".to_string());
                                        
                                        let goal_clone = goal.clone();
                                        
                                        let _ = wv.execute_script(&script, move |dom_json| {
                                            let dom: Vec<DOMItem> = serde_json::from_str(&dom_json).unwrap_or_default();
                                            
                                            // Call planner in background thread
                                            let hwnd_raw = STATE.with(|s| s.borrow().hwnd).map(|h| h.0 as usize).unwrap_or(0);
                                            if hwnd_raw != 0 {
                                                std::thread::spawn(move || {
                                                    let plan = call_agent_planner(&goal_clone, &dom);
                                                    
                                                    // Store plan for confirmation (on UI thread)
                                                    let plan_json = serde_json::json!({
                                                        "type": "agent-plan",
                                                        "plan": plan
                                                    }).to_string();
                                                    
                                                    // Send to UI thread via WM_APP+2
                                                    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
                                                    let boxed = Box::new(plan_json);
                                                    let ptr = Box::into_raw(boxed);
                                                    unsafe { PostMessageW(hwnd, WM_APP + 2, WPARAM(ptr as usize), LPARAM(0)); }
                                                });
                                            }
                                            Ok(())
                                        });
                                    }
                                }
                            });
                        },
                        _ => {}
                    }
                }
                Ok(())
            })?;
            
            STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.ai_sidebar_controller = Some(Rc::new(ctrl));
                state.layout_content();
            });
            Ok(())
        })?;
        Ok(())
    })
    .map_err(|e| anyhow::anyhow!("AI Sidebar Init Error: {:?}", e))?;
    Ok(())
}

fn create_isolated_tab(hwnd: HWND, tab_id: Uuid) -> anyhow::Result<()> {
    // Ephemeral Profile Path in TEMP (auto-cleaned by OS on reboot)
    let mut profile_path = std::env::temp_dir();
    profile_path.push("flextra_sessions");
    profile_path.push(format!("tab_{}", tab_id));

    webview2::Environment::builder()
        .with_user_data_folder(&profile_path)
        .with_additional_browser_arguments("--disable-features=msWebOOUI") 
        .build(move |env| {
            let env = env.map_err(|e| { error!("Tab Env error: {:?}", e); e })?;
            let winapi_hwnd = hwnd.0 as *mut winapi::shared::windef::HWND__;
            
            env.create_controller(winapi_hwnd, move |ctrl| {
                let ctrl = ctrl.map_err(|e| { error!("Tab ctrl error: {:?}", e); e })?;
                let webview = ctrl.get_webview()?;
                
                // Track this controller
                STATE.with(|s| {
                    let mut state = s.borrow_mut();
                    state.content_controllers.insert(tab_id, Rc::new(ctrl));
                    state.layout_content(); 
                    
                    // Initial Nav
                    if let Some(tab) = state.tabs.iter().find(|t| t.id == tab_id) {
                         if tab.url == "flxtra://privacy" {
                             // Find privacy dashboard (check multiple paths)
                             let cwd = std::env::current_dir().unwrap_or_default();
                             let path = if cwd.join("src/privacy_dashboard.html").exists() {
                                 cwd.join("src/privacy_dashboard.html")
                             } else if cwd.join("flxtra_browser/src/privacy_dashboard.html").exists() {
                                 cwd.join("flxtra_browser/src/privacy_dashboard.html")
                             } else {
                                 cwd.join("privacy_dashboard.html")
                             };
                             let url = format!("file:///{}", path.to_str().unwrap_or("").replace("\\", "/"));
                             let _ = webview.navigate(&url);
                             
                             // Attach Nuke Handler
                             webview.add_web_message_received(move |_, args| {
                                 if let Ok(msg) = args.try_get_web_message_as_string() {
                                     if msg == "nuke-session" {
                                         // Clean up and Close
                                         info!("NUKING SESSION DATA...");
                                         cleanup_session_data();
                                         std::process::exit(0);
                                     }
                                 }
                                 Ok(())
                             }).ok(); // Ignore error if fails
                         } else if !tab.url.is_empty() {
                             let _ = webview.navigate(&tab.url);
                         } else {
                             // Load landing page (check multiple paths)
                             let cwd = std::env::current_dir().unwrap_or_default();
                             let landing_path = if cwd.join("src/landing.html").exists() {
                                 cwd.join("src/landing.html")
                             } else if cwd.join("flxtra_browser/src/landing.html").exists() {
                                 cwd.join("flxtra_browser/src/landing.html")
                             } else {
                                 cwd.join("landing.html")
                             };
                             
                             if landing_path.exists() {
                                 let landing_url = format!("file:///{}", landing_path.to_str().unwrap_or("").replace("\\", "/"));
                                 let _ = webview.navigate(&landing_url);
                             }
                         }
                    }
                });

                // Title Sync
                webview.add_document_title_changed(move |wv| {
                    let title = wv.get_document_title().unwrap_or_else(|_| "New Tab".to_string());
                    STATE.with(|s| {
                        let mut state = s.borrow_mut();
                        if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == tab_id) {
                            tab.title = title;
                            state.sync_sidebar();
                        }
                    });
                    Ok(())
                })?;

                Ok(())
            })?;
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("Tab Creation Error: {:?}", e))?;

    Ok(())
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_SIZE => {
            STATE.with(|s| {
                let state = s.borrow();
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                let width = rect.right - rect.left;

                // Resize topbar
                if let Some(side) = &state.sidebar_controller {
                    let r = winapi::shared::windef::RECT { left: 0, top: 0, right: width, bottom: TOPBAR_HEIGHT };
                    let _ = side.put_bounds(r);
                }
                
                // Resize content -> All tabs handled in layout_content
                state.layout_content();
            });
            LRESULT(0)
        }
        WM_CLOSE => {
            // Privacy: Clean up all session data before exit
            cleanup_session_data();
            DestroyWindow(hwnd).ok();
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        msg if msg == WM_APP + 1 => {
            // AI Response Received
            let ptr = wparam.0 as *mut String;
            let text = unsafe { Box::from_raw(ptr) };
            
            STATE.with(|s| {
                if let Some(ctrl) = &s.borrow().ai_sidebar_controller {
                    if let Ok(wv) = ctrl.get_webview() {
                        let msg = serde_json::json!({
                            "type": "ai-response",
                            "content": *text
                        });
                        let _ = wv.post_web_message_as_json(&msg.to_string());
                    }
                }
            });
            LRESULT(0)
        }
        msg if msg == WM_APP + 2 => {
            // Agent Plan Received
            let ptr = wparam.0 as *mut String;
            let plan_json = unsafe { Box::from_raw(ptr) };
            
            STATE.with(|s| {
                if let Some(ctrl) = &s.borrow().ai_sidebar_controller {
                    if let Ok(wv) = ctrl.get_webview() {
                        let _ = wv.post_web_message_as_json(&plan_json);
                    }
                }
            });
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Delete all ephemeral tab profiles to ensure privacy
fn cleanup_session_data() {
    let session_dir = std::env::temp_dir().join("flextra_sessions");
    if session_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&session_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("tab_") {
                            let _ = std::fs::remove_dir_all(&path);
                            info!("Cleaned up session: {}", name);
                        }
                    }
                }
            }
        }
        // Also try to remove the parent directory if empty
        let _ = std::fs::remove_dir(&session_dir);
    }
}
