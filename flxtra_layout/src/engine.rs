//! Layout engine with security limits
use crate::box_model::{BoxType, EdgeSizes, LayoutBox, Rect};
use tracing::warn;

/// Maximum layout iterations to prevent infinite loops
const MAX_LAYOUT_ITERATIONS: usize = 1000;

/// Maximum layout tree depth
const MAX_LAYOUT_DEPTH: usize = 256;

pub struct LayoutEngine {
    viewport_width: f32,
    viewport_height: f32,
    default_font_size: f32,
    iteration_count: usize,
}

impl LayoutEngine {
    pub fn new(width: f32, height: f32) -> Self {
        Self { 
            viewport_width: width, 
            viewport_height: height, 
            default_font_size: 16.0,
            iteration_count: 0,
        }
    }

    pub fn layout(&mut self, root: &mut LayoutBox) -> bool {
        self.iteration_count = 0;
        let containing = Rect::new(0.0, 0.0, self.viewport_width, self.viewport_height);
        self.layout_box_safe(root, containing, 0)
    }

    fn layout_box_safe(&mut self, layout_box: &mut LayoutBox, containing: Rect, depth: usize) -> bool {
        // Security: Check iteration limit
        self.iteration_count += 1;
        if self.iteration_count > MAX_LAYOUT_ITERATIONS {
            warn!("Layout iteration limit exceeded");
            return false;
        }
        
        // Security: Check depth limit
        if depth > MAX_LAYOUT_DEPTH {
            warn!("Layout depth limit exceeded");
            return false;
        }

        match layout_box.box_type {
            BoxType::Block => self.layout_block(layout_box, containing, depth),
            BoxType::Inline | BoxType::Text => { 
                self.layout_inline(layout_box, containing); 
                true 
            }
            BoxType::InlineBlock => { 
                self.layout_inline_block(layout_box, containing); 
                true 
            }
            BoxType::Anonymous => true,
        }
    }

    fn layout_block(&mut self, layout_box: &mut LayoutBox, containing: Rect, depth: usize) -> bool {
        self.calculate_block_width(layout_box, containing);
        self.calculate_block_position(layout_box, containing);
        if !self.layout_children_safe(layout_box, depth) {
            return false;
        }
        self.calculate_height(layout_box);
        true
    }

    fn calculate_block_width(&self, layout_box: &mut LayoutBox, containing: Rect) {
        let width = containing.width 
            - layout_box.dimensions.margin.left - layout_box.dimensions.margin.right
            - layout_box.dimensions.padding.left - layout_box.dimensions.padding.right
            - layout_box.dimensions.border.left - layout_box.dimensions.border.right;
        layout_box.dimensions.content.width = width.max(0.0); // Prevent negative widths
    }

    fn calculate_block_position(&self, layout_box: &mut LayoutBox, containing: Rect) {
        let d = &mut layout_box.dimensions;
        d.margin = EdgeSizes { top: 8.0, right: 0.0, bottom: 8.0, left: 0.0 };
        d.content.x = containing.x + d.margin.left + d.border.left + d.padding.left;
        d.content.y = containing.y + containing.height + d.margin.top + d.border.top + d.padding.top;
    }

    fn layout_children_safe(&mut self, layout_box: &mut LayoutBox, depth: usize) -> bool {
        let mut height = 0.0;
        for child in &mut layout_box.children {
            let containing = Rect {
                x: layout_box.dimensions.content.x,
                y: layout_box.dimensions.content.y,
                width: layout_box.dimensions.content.width,
                height,
            };
            if !self.layout_box_safe(child, containing, depth + 1) {
                return false;
            }
            height += child.dimensions.margin_box().height;
        }
        true
    }

    fn calculate_height(&self, layout_box: &mut LayoutBox) {
        let height: f32 = layout_box.children.iter()
            .map(|c| c.dimensions.margin_box().height)
            .sum();
        layout_box.dimensions.content.height = height.max(20.0); // Minimum height
    }

    fn layout_inline(&self, layout_box: &mut LayoutBox, containing: Rect) {
        if let Some(text) = &layout_box.text {
            // Limit text measurement to prevent memory issues with very long text
            let len = text.len().min(10000);
            layout_box.dimensions.content.width = len as f32 * 8.0;
            layout_box.dimensions.content.height = self.default_font_size * 1.2;
        }
        layout_box.dimensions.content.x = containing.x;
        layout_box.dimensions.content.y = containing.y;
    }

    fn layout_inline_block(&self, layout_box: &mut LayoutBox, containing: Rect) {
        layout_box.dimensions.content.width = 100.0;
        layout_box.dimensions.content.height = 30.0;
        layout_box.dimensions.content.x = containing.x;
        layout_box.dimensions.content.y = containing.y;
    }
    
    /// Get iteration count for debugging
    pub fn iteration_count(&self) -> usize {
        self.iteration_count
    }
}
