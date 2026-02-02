//! Layout engine
use crate::box_model::{BoxType, EdgeSizes, LayoutBox, Rect};

pub struct LayoutEngine {
    viewport_width: f32,
    viewport_height: f32,
    default_font_size: f32,
}

impl LayoutEngine {
    pub fn new(width: f32, height: f32) -> Self {
        Self { viewport_width: width, viewport_height: height, default_font_size: 16.0 }
    }

    pub fn layout(&self, root: &mut LayoutBox) {
        let containing = Rect::new(0.0, 0.0, self.viewport_width, self.viewport_height);
        self.layout_box(root, containing);
    }

    fn layout_box(&self, layout_box: &mut LayoutBox, containing: Rect) {
        match layout_box.box_type {
            BoxType::Block => self.layout_block(layout_box, containing),
            BoxType::Inline | BoxType::Text => self.layout_inline(layout_box, containing),
            BoxType::InlineBlock => self.layout_inline_block(layout_box, containing),
            BoxType::Anonymous => {}
        }
    }

    fn layout_block(&self, layout_box: &mut LayoutBox, containing: Rect) {
        self.calculate_block_width(layout_box, containing);
        self.calculate_block_position(layout_box, containing);
        self.layout_children(layout_box);
        self.calculate_height(layout_box);
    }

    fn calculate_block_width(&self, layout_box: &mut LayoutBox, containing: Rect) {
        layout_box.dimensions.content.width = containing.width 
            - layout_box.dimensions.margin.left - layout_box.dimensions.margin.right
            - layout_box.dimensions.padding.left - layout_box.dimensions.padding.right
            - layout_box.dimensions.border.left - layout_box.dimensions.border.right;
    }

    fn calculate_block_position(&self, layout_box: &mut LayoutBox, containing: Rect) {
        let d = &mut layout_box.dimensions;
        d.margin = EdgeSizes { top: 8.0, right: 0.0, bottom: 8.0, left: 0.0 };
        d.content.x = containing.x + d.margin.left + d.border.left + d.padding.left;
        d.content.y = containing.y + containing.height + d.margin.top + d.border.top + d.padding.top;
    }

    fn layout_children(&self, layout_box: &mut LayoutBox) {
        let mut height = 0.0;
        for child in &mut layout_box.children {
            let containing = Rect {
                x: layout_box.dimensions.content.x,
                y: layout_box.dimensions.content.y,
                width: layout_box.dimensions.content.width,
                height,
            };
            self.layout_box(child, containing);
            height += child.dimensions.margin_box().height;
        }
    }

    fn calculate_height(&self, layout_box: &mut LayoutBox) {
        layout_box.dimensions.content.height = layout_box.children.iter()
            .map(|c| c.dimensions.margin_box().height)
            .sum();
        if layout_box.dimensions.content.height < 20.0 { layout_box.dimensions.content.height = 20.0; }
    }

    fn layout_inline(&self, layout_box: &mut LayoutBox, containing: Rect) {
        if let Some(text) = &layout_box.text {
            layout_box.dimensions.content.width = text.len() as f32 * 8.0; // Approx char width
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
}
