//! Windows native window implementation
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use tracing::info;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT, HBRUSH};

fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
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
            let class_name = to_wstring("AegisBrowser");
            
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                hInstance: instance.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hbrBackground: HBRUSH(16), // COLOR_WINDOW + 1
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
                None, None, instance, None,
            )?;
            
            info!("Created browser window: {}x{}", width, height);
            Ok(Self { hwnd, width, height, title: title.to_string() })
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
            WM_DESTROY => { PostQuitMessage(0); LRESULT(0) }
            WM_PAINT => { 
                let mut ps = PAINTSTRUCT::default();
                let _hdc = BeginPaint(hwnd, &mut ps);
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}
