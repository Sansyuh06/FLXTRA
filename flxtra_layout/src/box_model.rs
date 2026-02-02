//! CSS Box Model

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width && self.x + self.width > other.x &&
        self.y < other.y + other.height && self.y + self.height > other.y
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoxDimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

impl BoxDimensions {
    pub fn padding_box(&self) -> Rect {
        Rect {
            x: self.content.x - self.padding.left,
            y: self.content.y - self.padding.top,
            width: self.content.width + self.padding.left + self.padding.right,
            height: self.content.height + self.padding.top + self.padding.bottom,
        }
    }

    pub fn border_box(&self) -> Rect {
        let p = self.padding_box();
        Rect {
            x: p.x - self.border.left,
            y: p.y - self.border.top,
            width: p.width + self.border.left + self.border.right,
            height: p.height + self.border.top + self.border.bottom,
        }
    }

    pub fn margin_box(&self) -> Rect {
        let b = self.border_box();
        Rect {
            x: b.x - self.margin.left,
            y: b.y - self.margin.top,
            width: b.width + self.margin.left + self.margin.right,
            height: b.height + self.margin.top + self.margin.bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoxType {
    Block,
    Inline,
    InlineBlock,
    Anonymous,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBox {
    pub dimensions: BoxDimensions,
    pub box_type: BoxType,
    pub children: Vec<LayoutBox>,
    pub node_id: Option<u64>,
    pub text: Option<String>,
}

impl LayoutBox {
    pub fn new(box_type: BoxType) -> Self {
        Self {
            dimensions: BoxDimensions::default(),
            box_type,
            children: Vec::new(),
            node_id: None,
            text: None,
        }
    }

    pub fn block() -> Self { Self::new(BoxType::Block) }
    pub fn inline() -> Self { Self::new(BoxType::Inline) }
    pub fn text(content: String) -> Self {
        let mut b = Self::new(BoxType::Text);
        b.text = Some(content);
        b
    }

    pub fn add_child(&mut self, child: LayoutBox) { self.children.push(child); }
}
