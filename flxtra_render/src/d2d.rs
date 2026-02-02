//! GDI Renderer - Screen drawing using Windows GDI
//!
//! Simpler rendering using GDI instead of Direct2D

use crate::display_list::{DisplayCommand, DisplayList};
use flxtra_css::values::Color;
use flxtra_layout::box_model::Rect;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use tracing::{debug, info};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::*;

fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// GDI-based renderer
pub struct GdiRenderer {
    hwnd: Option<HWND>,
    width: u32,
    height: u32,
}

impl GdiRenderer {
    pub fn new() -> Self {
        info!("GDI renderer initialized");
        Self {
            hwnd: None,
            width: 0,
            height: 0,
        }
    }

    pub fn set_window(&mut self, hwnd: HWND, width: u32, height: u32) {
        self.hwnd = Some(hwnd);
        self.width = width;
        self.height = height;
        info!("GDI target set: {}x{}", width, height);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn render(&self, display_list: &DisplayList, bg_color: Color) {
        let hwnd = match self.hwnd {
            Some(h) => h,
            None => return,
        };

        unsafe {
            let hdc = GetDC(hwnd);
            if hdc.is_invalid() { return; }

            // Create double buffer
            let mem_dc = CreateCompatibleDC(hdc);
            let bitmap = CreateCompatibleBitmap(hdc, self.width as i32, self.height as i32);
            let old_bitmap = SelectObject(mem_dc, bitmap);

            // Clear background
            let bg_brush = CreateSolidBrush(COLORREF(self.to_colorref(bg_color)));
            let bg_rect = RECT { left: 0, top: 0, right: self.width as i32, bottom: self.height as i32 };
            FillRect(mem_dc, &bg_rect, bg_brush);
            let _ = DeleteObject(bg_brush);

            // Render commands
            for cmd in &display_list.commands {
                self.render_command(mem_dc, cmd);
            }

            // Copy to screen
            let _ = BitBlt(hdc, 0, 0, self.width as i32, self.height as i32, mem_dc, 0, 0, SRCCOPY);

            // Cleanup
            SelectObject(mem_dc, old_bitmap);
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(mem_dc);
            ReleaseDC(hwnd, hdc);
        }
    }

    unsafe fn render_command(&self, hdc: HDC, cmd: &DisplayCommand) {
        match cmd {
            DisplayCommand::SolidColor { color, rect } => {
                let brush = CreateSolidBrush(COLORREF(self.to_colorref(*color)));
                let r = self.to_gdi_rect(rect);
                FillRect(hdc, &r, brush);
                let _ = DeleteObject(brush);
            }
            DisplayCommand::Text { text, x, y, color, size } => {
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, COLORREF(self.to_colorref(*color)));
                
                // Create font
                let font = CreateFontW(
                    *size as i32,
                    0, 0, 0,
                    FW_NORMAL.0 as i32,
                    0, 0, 0,
                    DEFAULT_CHARSET.0 as u32,
                    OUT_DEFAULT_PRECIS.0 as u32,
                    CLIP_DEFAULT_PRECIS.0 as u32,
                    CLEARTYPE_QUALITY.0 as u32,
                    (FF_SWISS.0 | VARIABLE_PITCH.0) as u32,
                    windows::core::PCWSTR(to_wstring("Segoe UI").as_ptr()),
                );
                let old_font = SelectObject(hdc, font);
                
                let wide = to_wstring(text);
                let _ = TextOutW(hdc, *x as i32, (*y - size) as i32, &wide[..wide.len()-1]);
                
                SelectObject(hdc, old_font);
                let _ = DeleteObject(font);
            }
            DisplayCommand::Border { rect, color, width } => {
                let pen = CreatePen(PS_SOLID, *width as i32, COLORREF(self.to_colorref(*color)));
                let old_pen = SelectObject(hdc, pen);
                let null_brush = GetStockObject(NULL_BRUSH);
                let old_brush = SelectObject(hdc, null_brush);
                
                let _ = Rectangle(hdc, rect.x as i32, rect.y as i32, 
                    (rect.x + rect.width) as i32, (rect.y + rect.height) as i32);
                
                SelectObject(hdc, old_brush);
                SelectObject(hdc, old_pen);
                let _ = DeleteObject(pen);
            }
            DisplayCommand::Image { url, rect } => {
                debug!("Image placeholder: {}", url);
                // Draw placeholder
                let brush = CreateSolidBrush(COLORREF(0x404040));
                let r = self.to_gdi_rect(rect);
                FillRect(hdc, &r, brush);
                let _ = DeleteObject(brush);
            }
        }
    }

    fn to_colorref(&self, color: Color) -> u32 {
        // GDI uses BGR format
        ((color.b as u32) << 16) | ((color.g as u32) << 8) | (color.r as u32)
    }

    fn to_gdi_rect(&self, rect: &Rect) -> RECT {
        RECT {
            left: rect.x as i32,
            top: rect.y as i32,
            right: (rect.x + rect.width) as i32,
            bottom: (rect.y + rect.height) as i32,
        }
    }
}

impl Default for GdiRenderer {
    fn default() -> Self { Self::new() }
}
