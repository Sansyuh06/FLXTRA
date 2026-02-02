//! Painter - generates display list from layout tree
use crate::display_list::{DisplayCommand, DisplayList};
use flxtra_css::values::Color;
use flxtra_layout::box_model::{BoxType, LayoutBox};

pub struct Painter;

impl Painter {
    pub fn paint(layout_root: &LayoutBox) -> DisplayList {
        let mut list = DisplayList::new();
        Self::paint_box(&mut list, layout_root);
        list
    }

    fn paint_box(list: &mut DisplayList, layout_box: &LayoutBox) {
        Self::paint_background(list, layout_box);
        Self::paint_borders(list, layout_box);
        Self::paint_text(list, layout_box);
        for child in &layout_box.children { Self::paint_box(list, child); }
    }

    fn paint_background(list: &mut DisplayList, layout_box: &LayoutBox) {
        let rect = layout_box.dimensions.border_box();
        if rect.width > 0.0 && rect.height > 0.0 {
            let color = match layout_box.box_type {
                BoxType::Block => Color::WHITE,
                _ => Color::TRANSPARENT,
            };
            if color.a > 0 { list.fill_rect(rect, color); }
        }
    }

    fn paint_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
        let b = &layout_box.dimensions.border;
        if b.top > 0.0 || b.right > 0.0 || b.bottom > 0.0 || b.left > 0.0 {
            list.draw_border(layout_box.dimensions.border_box(), Color::BLACK, b.top);
        }
    }

    fn paint_text(list: &mut DisplayList, layout_box: &LayoutBox) {
        if let Some(text) = &layout_box.text {
            let rect = &layout_box.dimensions.content;
            list.draw_text(text.clone(), rect.x, rect.y + 14.0, Color::BLACK, 16.0);
        }
    }
}
