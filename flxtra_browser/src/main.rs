//! Flxtra Browser - Comet Clone (Dual-WebView + Tab Virtualization)
//!
//! Two webviews: 
//! 1. Sidebar (UI) - Shared Environment
//! 2. Content Tabs - ISOLATED Environments (Unique UserDataFolder per tab)

use flxtra_filter::FilterEngine;
use once_cell::sync::Lazy;
use std::sync::Arc;
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

// Global filter engine
static FILTER_ENGINE: Lazy<Arc<FilterEngine>> = Lazy::new(|| Arc::new(FilterEngine::new()));

// AI Service - Ollama Integration
fn call_ai(prompt: &str, action: &str, context: &str) -> String {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .ok();
    
    let full_prompt = match action {
        "summarize" => format!("Summarize this webpage content in 3 concise bullet points:\n\n{}", prompt),
        "explain" => format!("Explain this webpage content in simple terms that a 12-year-old could understand:\n\n{}", prompt),
        "keypoints" => format!("Extract the 5 most important facts from this content as a numbered list:\n\n{}", prompt),
        "ask" => format!("Context from webpage:\n{}\n\nQuestion: {}\n\nAnswer the question based on the context above:", context, prompt),
        _ => format!("Analyze this content:\n\n{}", prompt),
    };
    
    // Try Ollama first (local)
    if let Some(ref c) = client {
        if let Ok(res) = c.post("http://localhost:11434/api/generate")
            .json(&serde_json::json!({
                "model": "mistral",
                "prompt": full_prompt,
                "stream": false
            }))
            .send() 
        {
            if let Ok(body) = res.json::<serde_json::Value>() {
                if let Some(response) = body["response"].as_str() {
                    return response.to_string();
                }
            }
        }
    }
    
    if action == "ask" {
        return "Unable to answer. Please ensure Ollama is running locally.".to_string();
    }
    
    // Fallback: Simple extractive summary
    let sentences: Vec<&str> = prompt.split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| s.len() > 20)
        .take(3)
        .collect();
    
    if sentences.is_empty() {
        "Unable to analyze content. Please ensure Ollama is running locally.".to_string()
    } else {
        format!("• {}", sentences.join("\n• "))
    }
}

// Agent Planner - Ollama ReAct
fn call_agent_planner(goal: &str, dom: &[DOMItem]) -> Option<AgentPlan> {
    let dom_desc = dom.iter()
        .take(50) // Limit context
        .map(|d| format!("[{}] {} '{}'", d.id, d.tag, d.label))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "Goal: \"{}\"\n\nVisible Interactive Elements:\n{}\n\nReturn the NEXT step as a JSON object with fields: action (click/type), target (id), value (optional), description (short reason). JSON ONLY.",
        goal, dom_desc
    );

    let client = reqwest::blocking::Client::new();
    if let Ok(res) = client.post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "mistral",
            "prompt": prompt,
            "stream": false,
            "format": "json"
        }))
        .send() 
    {
        if let Ok(body) = res.json::<serde_json::Value>() {
            if let Some(resp_str) = body["response"].as_str() {
                // Try parsing JSON
                if let Ok(plan) = serde_json::from_str::<AgentPlan>(resp_str) {
                    return Some(plan);
                }
            }
        }
    }
    None
}

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

#[derive(Serialize, Deserialize, Debug)]
struct DOMItem {
    id: u32,
    tag: String,
    label: String,
    #[serde(default)]
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AgentPlan {
    action: String, // click, type, scroll
    target: u32,
    value: Option<String>,
    description: String,
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
    // Map Tab ID -> Controller. Each controller has its own Environment (Profile)
    content_controllers: HashMap<Uuid, Rc<Controller>>,
    tabs: Vec<TabInfo>,
    active_tab_id: Uuid,
    pending_plan: Option<AgentPlan>,
}

impl BrowserState {
    fn new() -> Self {
        let initial_tab = TabInfo::new();
        let state = Self {
            hwnd: None,
            sidebar_controller: None,
            content_controllers: HashMap::new(),
            tabs: vec![initial_tab.clone()],
            active_tab_id: initial_tab.id,
            pending_plan: None,
        };
        // state.load_session(); // Disabled temporarily while refactoring structure
        state
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
        
        let visible_rect = winapi::shared::windef::RECT {
            left: 0, top: TOPBAR_HEIGHT,
            right: width, bottom: height
        };
        
        // Hide others (move offscreen/zero size)
        // Note: For true performance we might want to suspend them, but resizing is safer for now
        let _hidden_rect = winapi::shared::windef::RECT { left: 0, top: 0, right: 0, bottom: 0 };

        for (id, ctrl) in &self.content_controllers {
            if *id == self.active_tab_id {
                let _ = ctrl.put_bounds(visible_rect);
                let _ = ctrl.put_is_visible(true);
            } else {
                let _ = ctrl.put_bounds(visible_rect); // Keep bounds but hide
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
            
            // Load Sidebar
            let sidebar_path = std::env::current_dir().unwrap_or_default().join("flxtra_browser/src/sidebar.html");
            let sidebar_url = format!("file:///{}", sidebar_path.to_str().unwrap().replace("\\", "/"));
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
                                state.tabs.last_mut().unwrap().active = true;
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
                                         }).unwrap();
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
                                        }).unwrap();
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
                                        }).unwrap();
                                    }
                                }
                            });
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
                                                    "document.querySelector('[data-agent-id=\"{}\"]').click();", 
                                                    p.target
                                                ),
                                                "type" => format!(
                                                    "let el = document.querySelector('[data-agent-id=\"{}\"]'); if(el) {{ el.value = '{}'; el.dispatchEvent(new Event('input', {{ bubbles: true }})); }}", 
                                                    p.target, p.value.unwrap_or_default()
                                                ),
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

fn create_isolated_tab(hwnd: HWND, tab_id: Uuid) -> anyhow::Result<()> {
    // Unique Profile Path per Tab
    let mut profile_path = std::env::current_dir()?;
    profile_path.push("user_data");
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
                         if !tab.url.is_empty() {
                             let _ = webview.navigate(&tab.url);
                         } else {
                             // Load landing page for new tabs (from exe directory)
                             if let Ok(exe_path) = std::env::current_exe() {
                                 if let Some(exe_dir) = exe_path.parent() {
                                     let landing_path = exe_dir.join("landing.html");
                                     let landing_url = format!("file:///{}", landing_path.to_str().unwrap().replace("\\", "/"));
                                     let _ = webview.navigate(&landing_url);
                                 }
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
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
