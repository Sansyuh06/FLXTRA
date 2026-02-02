//! Flxtra Browser - Comet Clone (Dual-WebView Architecture)
//!
//! Two webviews: 
//! 1. Sidebar (UI, Vertical Tabs, Navigation)
//! 2. Content (Active Web Page)

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
use serde::{Serialize, Deserialize};

// Helper to convert string to Windows wide string
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

// Global filter engine
static FILTER_ENGINE: Lazy<Arc<FilterEngine>> = Lazy::new(|| Arc::new(FilterEngine::new()));

// Sidebar width
const SIDEBAR_WIDTH: i32 = 260;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TabInfo {
    title: String,
    url: String,
    favicon: Option<String>,
    active: bool,
}

struct BrowserState {
    sidebar_controller: Option<Rc<Controller>>,
    content_controller: Option<Rc<Controller>>,
    tabs: Vec<TabInfo>,
    active_tab_idx: usize,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            sidebar_controller: None,
            content_controller: None,
            tabs: vec![TabInfo {
                title: "New Tab".to_string(),
                url: "".to_string(),
                favicon: Some("✨".to_string()),
                active: true,
            }],
            active_tab_idx: 0,
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
}

thread_local! {
    static STATE: RefCell<BrowserState> = RefCell::new(BrowserState::new());
}

fn main() -> anyhow::Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env().add_directive("Flxtra=info".parse()?)).init();
    info!("Starting Flxtra Comet Edition...");

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
            PCWSTR(to_wstring("Flxtra Browser").as_ptr()),
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

        // Initialize WebViews
        init_webviews(hwnd)?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

fn init_webviews(hwnd: HWND) -> anyhow::Result<()> {
    // 1. Initialize Sidebar
    webview2::Environment::builder().build(move |env| {
        let env = env.map_err(|e| { error!("Env error: {:?}", e); e })?;
        let env = Rc::new(env);
        let env_clone = env.clone();
        let winapi_hwnd = hwnd.0 as *mut winapi::shared::windef::HWND__;
        
        env.create_controller(winapi_hwnd, move |sidebar_ctrl| {
            let sidebar_ctrl = sidebar_ctrl.map_err(|e| { error!("Sidebar ctrl error: {:?}", e); e })?;
            let sidebar_wv = sidebar_ctrl.get_webview()?;
            
            // Layout Sidebar
            let mut rect = RECT::default();
            unsafe { GetClientRect(hwnd, &mut rect); }
            let side_rect = winapi::shared::windef::RECT {
                left: 0, top: 0,
                right: SIDEBAR_WIDTH,
                bottom: rect.bottom - rect.top,
            };
            sidebar_ctrl.put_bounds(side_rect)?;

            // Load Sidebar UI
            let sidebar_path = std::env::current_dir().unwrap_or_default().join("flxtra_browser/src/sidebar.html");
            let sidebar_url = format!("file:///{}", sidebar_path.to_str().unwrap().replace("\\", "/"));
            sidebar_wv.navigate(&sidebar_url)?;

            // Handle Messages from Sidebar
            sidebar_wv.add_web_message_received(move |_, args| {
                let msg = args.get_web_message_as_json()?;
                let val: serde_json::Value = serde_json::from_str(&msg).unwrap_or_default();
                
                if let Some(cmd) = val["command"].as_str() {
                    match cmd {
                        "navigate" => {
                            if let Some(url) = val["data"].as_str() {
                                STATE.with(|s| {
                                    if let Some(content) = &s.borrow().content_controller {
                                        let final_url = if url.contains('.') && !url.contains(' ') {
                                            if url.starts_with("http") { url.to_string() } else { format!("https://{}", url) }
                                        } else {
                                            format!("https://duckduckgo.com/?q={}", url)
                                        };
                                        let _ = content.get_webview().unwrap().navigate(&final_url);
                                    }
                                });
                            }
                        }
                        "back" => STATE.with(|s| if let Some(c) = &s.borrow().content_controller { let _ = c.get_webview().unwrap().go_back(); }),
                        "forward" => STATE.with(|s| if let Some(c) = &s.borrow().content_controller { let _ = c.get_webview().unwrap().go_forward(); }),
                        "reload" => STATE.with(|s| if let Some(c) = &s.borrow().content_controller { let _ = c.get_webview().unwrap().reload(); }),
                        _ => debug!("Unhandled sidebar command: {}", cmd),
                    }
                }
                Ok(())
            })?;

            let sidebar_ctrl_rc = Rc::new(sidebar_ctrl);
            STATE.with(|s| {
                s.borrow_mut().sidebar_controller = Some(sidebar_ctrl_rc.clone());
            });

            // 2. Initialize Content
            env_clone.create_controller(winapi_hwnd, move |content_ctrl| {
                let content_ctrl = content_ctrl.map_err(|e| { error!("Content ctrl error: {:?}", e); e })?;
                let content_wv = content_ctrl.get_webview()?;
                
                // Layout Content
                let mut rect = RECT::default();
                unsafe { GetClientRect(hwnd, &mut rect); }
                let cont_rect = winapi::shared::windef::RECT {
                    left: SIDEBAR_WIDTH, top: 0,
                    right: rect.right - rect.left,
                    bottom: rect.bottom - rect.top,
                };
                content_ctrl.put_bounds(cont_rect)?;

                // Sync page title/URL back to sidebar
                content_wv.add_document_title_changed(move |wv| {
                    let title = wv.get_document_title().unwrap_or_else(|_| "New Tab".to_string());
                    STATE.with(|s| {
                        let mut state = s.borrow_mut();
                        let idx = state.active_tab_idx;
                        state.tabs[idx].title = title;
                        state.sync_sidebar();
                    });
                    Ok(())
                })?;

                STATE.with(|s| s.borrow_mut().content_controller = Some(Rc::new(content_ctrl)));
                
                info!("Dual WebViews initialized successfully.");
                Ok(())
            })?;
            Ok(())
        })?;
        Ok(())
    })
    .map_err(|e| anyhow::anyhow!("WebView2 build error: {:?}", e))?;
    Ok(())
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_SIZE => {
            STATE.with(|s| {
                let state = s.borrow();
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                let height = rect.bottom - rect.top;
                let width = rect.right - rect.left;

                if let Some(side) = &state.sidebar_controller {
                    let r = winapi::shared::windef::RECT { left: 0, top: 0, right: SIDEBAR_WIDTH, bottom: height };
                    let _ = side.put_bounds(r);
                }
                if let Some(content) = &state.content_controller {
                    let r = winapi::shared::windef::RECT { left: SIDEBAR_WIDTH, top: 0, right: width, bottom: height };
                    let _ = content.put_bounds(r);
                }
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
