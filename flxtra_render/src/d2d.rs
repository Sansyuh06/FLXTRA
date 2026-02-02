//! Direct2D Renderer - Actual screen drawing
//!
//! Renders display list commands to the Windows window using Direct2D

use crate::display_list::{DisplayCommand, DisplayList};
use flxtra_css::values::Color;
use flxtra_layout::box_model::Rect;
use std::ptr;
use tracing::{debug, info};
use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Gdi::*;

/// Direct2D Renderer
pub struct D2DRenderer {
    factory: ID2D1Factory,
    write_factory: IDWriteFactory,
    render_target: Option<ID2D1HwndRenderTarget>,
    text_format: Option<IDWriteTextFormat>,
}

impl D2DRenderer {
    pub fn new() -> windows::core::Result<Self> {
        unsafe {
            let factory: ID2D1Factory = D2D1CreateFactory(
                D2D1_FACTORY_TYPE_SINGLE_THREADED,
                None,
            )?;

            let write_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;

            info!("Direct2D renderer initialized");

            Ok(Self {
                factory,
                write_factory,
                render_target: None,
                text_format: None,
            })
        }
    }

    pub fn create_render_target(&mut self, hwnd: HWND, width: u32, height: u32) -> windows::core::Result<()> {
        unsafe {
            let render_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: D2D_SIZE_U { width, height },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let render_target = self.factory.CreateHwndRenderTarget(&render_props, &hwnd_props)?;
            self.render_target = Some(render_target);

            // Create default text format
            let text_format = self.write_factory.CreateTextFormat(
                PCWSTR::from_raw("Segoe UI\0".encode_utf16().collect::<Vec<u16>>().as_ptr()),
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                16.0,
                PCWSTR::from_raw("en-us\0".encode_utf16().collect::<Vec<u16>>().as_ptr()),
            )?;
            self.text_format = Some(text_format);

            info!("Render target created: {}x{}", width, height);
            Ok(())
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(ref target) = self.render_target {
            unsafe {
                let size = D2D_SIZE_U { width, height };
                let _ = target.Resize(&size);
            }
        }
    }

    pub fn render(&self, display_list: &DisplayList, bg_color: Color) {
        let target = match &self.render_target {
            Some(t) => t,
            None => return,
        };

        unsafe {
            target.BeginDraw();

            // Clear background
            let clear_color = self.to_d2d_color(bg_color);
            target.Clear(Some(&clear_color));

            // Render each command
            for cmd in &display_list.commands {
                self.render_command(target, cmd);
            }

            let _ = target.EndDraw(None, None);
        }
    }

    unsafe fn render_command(&self, target: &ID2D1HwndRenderTarget, cmd: &DisplayCommand) {
        match cmd {
            DisplayCommand::SolidColor { color, rect } => {
                let brush = target.CreateSolidColorBrush(&self.to_d2d_color(*color), None).ok();
                if let Some(brush) = brush {
                    let d2d_rect = self.to_d2d_rect(rect);
                    target.FillRectangle(&d2d_rect, &brush);
                }
            }
            DisplayCommand::Text { text, x, y, color, size } => {
                let brush = target.CreateSolidColorBrush(&self.to_d2d_color(*color), None).ok();
                if let (Some(brush), Some(format)) = (brush, &self.text_format) {
                    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                    let layout_rect = D2D_RECT_F {
                        left: *x,
                        top: *y - size,
                        right: *x + 1000.0,
                        bottom: *y + size,
                    };
                    target.DrawText(
                        &wide[..wide.len()-1],
                        format,
                        &layout_rect,
                        &brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                        DWRITE_MEASURING_MODE_NATURAL,
                    );
                }
            }
            DisplayCommand::Border { rect, color, width } => {
                let brush = target.CreateSolidColorBrush(&self.to_d2d_color(*color), None).ok();
                if let Some(brush) = brush {
                    let d2d_rect = self.to_d2d_rect(rect);
                    target.DrawRectangle(&d2d_rect, &brush, *width, None);
                }
            }
            DisplayCommand::Image { url, rect } => {
                // TODO: Image rendering
                debug!("Image placeholder: {}", url);
            }
        }
    }

    fn to_d2d_color(&self, color: Color) -> D2D1_COLOR_F {
        D2D1_COLOR_F {
            r: color.r as f32 / 255.0,
            g: color.g as f32 / 255.0,
            b: color.b as f32 / 255.0,
            a: color.a as f32 / 255.0,
        }
    }

    fn to_d2d_rect(&self, rect: &Rect) -> D2D_RECT_F {
        D2D_RECT_F {
            left: rect.x,
            top: rect.y,
            right: rect.x + rect.width,
            bottom: rect.y + rect.height,
        }
    }
}

impl Default for D2DRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create D2D renderer")
    }
}
