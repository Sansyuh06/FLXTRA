//! CSS value types

use serde::{Deserialize, Serialize};

/// CSS length value
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Length {
    Px(f32),
    Em(f32),
    Rem(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
    Auto,
    Zero,
}

impl Length {
    /// Convert to pixels given context
    pub fn to_px(&self, parent_px: f32, root_font_size: f32, viewport_width: f32, viewport_height: f32) -> f32 {
        match self {
            Length::Px(v) => *v,
            Length::Em(v) => v * parent_px,
            Length::Rem(v) => v * root_font_size,
            Length::Percent(v) => v / 100.0 * parent_px,
            Length::Vw(v) => v / 100.0 * viewport_width,
            Length::Vh(v) => v / 100.0 * viewport_height,
            Length::Auto => 0.0,
            Length::Zero => 0.0,
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, Length::Auto)
    }
}

impl Default for Length {
    fn default() -> Self {
        Length::Zero
    }
}

/// CSS color value
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Self::rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Parse named color
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "transparent" => Some(Self::TRANSPARENT),
            "black" => Some(Self::BLACK),
            "white" => Some(Self::WHITE),
            "red" => Some(Self::rgb(255, 0, 0)),
            "green" => Some(Self::rgb(0, 128, 0)),
            "blue" => Some(Self::rgb(0, 0, 255)),
            "yellow" => Some(Self::rgb(255, 255, 0)),
            "cyan" => Some(Self::rgb(0, 255, 255)),
            "magenta" => Some(Self::rgb(255, 0, 255)),
            "gray" | "grey" => Some(Self::rgb(128, 128, 128)),
            "silver" => Some(Self::rgb(192, 192, 192)),
            "maroon" => Some(Self::rgb(128, 0, 0)),
            "olive" => Some(Self::rgb(128, 128, 0)),
            "purple" => Some(Self::rgb(128, 0, 128)),
            "teal" => Some(Self::rgb(0, 128, 128)),
            "navy" => Some(Self::rgb(0, 0, 128)),
            "orange" => Some(Self::rgb(255, 165, 0)),
            "pink" => Some(Self::rgb(255, 192, 203)),
            _ => None,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// Display type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Display {
    None,
    Block,
    Inline,
    InlineBlock,
    Flex,
    Grid,
    Contents,
}

impl Default for Display {
    fn default() -> Self {
        Display::Inline
    }
}

/// Position type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

impl Default for Position {
    fn default() -> Self {
        Position::Static
    }
}

/// Float
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Float {
    None,
    Left,
    Right,
}

impl Default for Float {
    fn default() -> Self {
        Float::None
    }
}

/// Overflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

impl Default for Overflow {
    fn default() -> Self {
        Overflow::Visible
    }
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Left
    }
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontWeight(pub u16);

impl FontWeight {
    pub const NORMAL: FontWeight = FontWeight(400);
    pub const BOLD: FontWeight = FontWeight(700);
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle::Normal
    }
}

/// Border style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::None
    }
}

/// Box side (for margin, padding, border)
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct BoxSides<T: Default + Copy> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Default + Copy> BoxSides<T> {
    pub fn all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn vertical_horizontal(vertical: T, horizontal: T) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}
