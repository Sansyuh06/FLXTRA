//! Render Engine - coordinates rendering pipeline
use aegis_css::ComputedStyle;
use aegis_layout::LayoutBox;
use aegis_render::DisplayList;

pub struct RenderEngine {
    viewport_width: f32,
    viewport_height: f32,
}

impl RenderEngine {
    pub fn new(width: f32, height: f32) -> Self {
        Self { viewport_width: width, viewport_height: height }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    pub fn viewport(&self) -> (f32, f32) {
        (self.viewport_width, self.viewport_height)
    }
}
