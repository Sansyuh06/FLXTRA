//! Computed style values

use crate::values::*;
use serde::{Deserialize, Serialize};

/// Computed style for an element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedStyle {
    // Display
    pub display: Display,
    pub position: Position,
    pub float: Float,
    pub visibility: bool,
    pub opacity: f32,

    // Box model
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,

    // Margin
    pub margin: BoxSides<Length>,

    // Padding
    pub padding: BoxSides<Length>,

    // Border
    pub border_width: BoxSides<Length>,
    pub border_style: BoxSides<BorderStyle>,
    pub border_color: BoxSides<Color>,
    pub border_radius: Length,

    // Colors
    pub color: Color,
    pub background_color: Color,

    // Typography
    pub font_family: String,
    pub font_size: Length,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub line_height: Length,
    pub text_align: TextAlign,

    // Positioning
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
    pub z_index: i32,

    // Overflow
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,

    // Flex
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Length,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Inline,
            position: Position::Static,
            float: Float::None,
            visibility: true,
            opacity: 1.0,

            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Zero,
            min_height: Length::Zero,
            max_width: Length::Auto,
            max_height: Length::Auto,

            margin: BoxSides::all(Length::Zero),
            padding: BoxSides::all(Length::Zero),

            border_width: BoxSides::all(Length::Zero),
            border_style: BoxSides::all(BorderStyle::None),
            border_color: BoxSides::all(Color::BLACK),
            border_radius: Length::Zero,

            color: Color::BLACK,
            background_color: Color::TRANSPARENT,

            font_family: "sans-serif".to_string(),
            font_size: Length::Px(16.0),
            font_weight: FontWeight::NORMAL,
            font_style: FontStyle::Normal,
            line_height: Length::Px(20.0),
            text_align: TextAlign::Left,

            top: Length::Auto,
            right: Length::Auto,
            bottom: Length::Auto,
            left: Length::Auto,
            z_index: 0,

            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,

            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Length::Auto,
        }
    }
}

impl ComputedStyle {
    /// Create default style for a block element
    pub fn block_default() -> Self {
        Self {
            display: Display::Block,
            ..Default::default()
        }
    }

    /// Create default style for body element
    pub fn body_default() -> Self {
        Self {
            display: Display::Block,
            margin: BoxSides::all(Length::Px(8.0)), // Default browser margin
            ..Default::default()
        }
    }

    /// Check if element is visible
    pub fn is_visible(&self) -> bool {
        self.visibility
            && self.opacity > 0.0
            && self.display != Display::None
    }

    /// Check if element creates a new stacking context
    pub fn creates_stacking_context(&self) -> bool {
        self.z_index != 0
            || self.opacity < 1.0
            || self.position == Position::Fixed
            || self.position == Position::Sticky
    }

    /// Check if element is positioned
    pub fn is_positioned(&self) -> bool {
        self.position != Position::Static
    }

    /// Get content box dimensions
    pub fn content_width(&self, viewport_width: f32) -> f32 {
        self.width.to_px(viewport_width, 16.0, viewport_width, 0.0)
    }

    pub fn content_height(&self, viewport_height: f32) -> f32 {
        self.height.to_px(viewport_height, 16.0, 0.0, viewport_height)
    }
}

/// Default styles for HTML elements
pub fn default_style_for_tag(tag: &str) -> ComputedStyle {
    match tag.to_lowercase().as_str() {
        // Block elements
        "html" | "body" | "div" | "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" => {
            ComputedStyle::block_default()
        }
        "p" => ComputedStyle {
            display: Display::Block,
            margin: BoxSides::vertical_horizontal(Length::Em(1.0), Length::Zero),
            ..Default::default()
        },
        "h1" => ComputedStyle {
            display: Display::Block,
            font_size: Length::Em(2.0),
            font_weight: FontWeight::BOLD,
            margin: BoxSides::vertical_horizontal(Length::Em(0.67), Length::Zero),
            ..Default::default()
        },
        "h2" => ComputedStyle {
            display: Display::Block,
            font_size: Length::Em(1.5),
            font_weight: FontWeight::BOLD,
            margin: BoxSides::vertical_horizontal(Length::Em(0.83), Length::Zero),
            ..Default::default()
        },
        "h3" => ComputedStyle {
            display: Display::Block,
            font_size: Length::Em(1.17),
            font_weight: FontWeight::BOLD,
            margin: BoxSides::vertical_horizontal(Length::Em(1.0), Length::Zero),
            ..Default::default()
        },
        "ul" | "ol" => ComputedStyle {
            display: Display::Block,
            margin: BoxSides::vertical_horizontal(Length::Em(1.0), Length::Zero),
            padding: BoxSides {
                left: Length::Px(40.0),
                ..BoxSides::all(Length::Zero)
            },
            ..Default::default()
        },
        "li" => ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
        "pre" | "code" => ComputedStyle {
            font_family: "monospace".to_string(),
            ..Default::default()
        },
        "a" => ComputedStyle {
            color: Color::rgb(0, 0, 238),
            ..Default::default()
        },
        "strong" | "b" => ComputedStyle {
            font_weight: FontWeight::BOLD,
            ..Default::default()
        },
        "em" | "i" => ComputedStyle {
            font_style: FontStyle::Italic,
            ..Default::default()
        },
        "img" | "video" | "canvas" | "iframe" => ComputedStyle {
            display: Display::InlineBlock,
            ..Default::default()
        },
        "table" => ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
        "input" | "button" | "select" | "textarea" => ComputedStyle {
            display: Display::InlineBlock,
            ..Default::default()
        },
        "br" | "hr" => ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
        "script" | "style" | "template" | "noscript" => ComputedStyle {
            display: Display::None,
            ..Default::default()
        },
        _ => ComputedStyle::default(),
    }
}
