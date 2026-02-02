//! Windows native window implementation with event handling
use std::cell::RefCell;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use tracing::info;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT, HBRUSH, InvalidateRect};

fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// Event types from the window
#[derive(Debug, Clone)]
pub enum WindowEvent {
    Click { x: f32, y: f32 },
    KeyPress { char: char },
    Resize { width: u32, height: u32 },
    Paint,
    Close,
}

// Thread-local storage for the event callback
thread_local! {
    static EVENT_CALLBACK: RefCell<Option<Box<dyn FnMut(WindowEvent)>>> = RefCell::new(None);
    static CURRENT_HWND: RefCell<HWND> = RefCell::new(HWND::default());
}

pub struct BrowserWindow {
    pub hwnd: HWND,
    pub width: u32,
    pub height: u32,
    pub title: String,
}

impl BrowserWindow {
    pub fn new(title: &str, width: u32, height: u32) -> windows::core::Result<Self> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            let class_name = to_wstring("FlxtraBrowser");
            
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                hInstance: instance.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hbrBackground: HBRUSH(1), // Black background
                ..Default::default()
            };
            
            RegisterClassExW(&wc);
            
            let title_wide = to_wstring(title);
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(title_wide.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT, CW_USEDEFAULT,
                width as i32, height as i32,
                HWND::default(), None, instance, None,
            );
            
            CURRENT_HWND.with(|h| *h.borrow_mut() = hwnd);
            
            info!("Created browser window: {}x{}", width, height);
            Ok(Self { hwnd, width, height, title: title.to_string() })
        }
    }

    /// Set event callback
    pub fn set_event_callback<F: FnMut(WindowEvent) + 'static>(&self, callback: F) {
        EVENT_CALLBACK.with(|cb| {
            *cb.borrow_mut() = Some(Box::new(callback));
        });
    }

    /// Request a repaint
    pub fn request_paint(&self) {
        unsafe {
            let _ = InvalidateRect(self.hwnd, None, false);
        }
    }

    pub fn run_event_loop(&self) {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        let title_wide = to_wstring(title);
        unsafe { let _ = SetWindowTextW(self.hwnd, PCWSTR(title_wide.as_ptr())); }
    }

    unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_DESTROY => { 
                Self::send_event(WindowEvent::Close);
                PostQuitMessage(0); 
                LRESULT(0) 
            }
            WM_PAINT => { 
                let mut ps = PAINTSTRUCT::default();
                let _hdc = BeginPaint(hwnd, &mut ps);
                Self::send_event(WindowEvent::Paint);
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                let x = (lparam.0 & 0xFFFF) as i16 as f32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;
                Self::send_event(WindowEvent::Click { x, y });
                // Request repaint after click
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_CHAR => {
                let char_code = wparam.0 as u32;
                if let Some(c) = char::from_u32(char_code) {
                    Self::send_event(WindowEvent::KeyPress { char: c });
                    let _ = InvalidateRect(hwnd, None, false);
                }
                LRESULT(0)
            }
            WM_SIZE => {
                let width = (lparam.0 & 0xFFFF) as u32;
                let height = ((lparam.0 >> 16) & 0xFFFF) as u32;
                Self::send_event(WindowEvent::Resize { width, height });
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }

    fn send_event(event: WindowEvent) {
        EVENT_CALLBACK.with(|cb| {
            if let Some(ref mut callback) = *cb.borrow_mut() {
                callback(event);
            }
        });
    }
}
