//! CSS properties

use crate::values::*;
use serde::{Deserialize, Serialize};

/// CSS property declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
    pub important: bool,
}

/// CSS property value enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    // Layout
    Display(Display),
    Position(Position),
    Float(Float),
    
    // Box model
    Width(Length),
    Height(Length),
    MinWidth(Length),
    MinHeight(Length),
    MaxWidth(Length),
    MaxHeight(Length),
    
    // Margins
    MarginTop(Length),
    MarginRight(Length),
    MarginBottom(Length),
    MarginLeft(Length),
    
    // Padding
    PaddingTop(Length),
    PaddingRight(Length),
    PaddingBottom(Length),
    PaddingLeft(Length),
    
    // Border
    BorderTopWidth(Length),
    BorderRightWidth(Length),
    BorderBottomWidth(Length),
    BorderLeftWidth(Length),
    BorderTopStyle(BorderStyle),
    BorderRightStyle(BorderStyle),
    BorderBottomStyle(BorderStyle),
    BorderLeftStyle(BorderStyle),
    BorderTopColor(Color),
    BorderRightColor(Color),
    BorderBottomColor(Color),
    BorderLeftColor(Color),
    BorderRadius(Length),
    
    // Colors
    Color(Color),
    BackgroundColor(Color),
    
    // Typography
    FontFamily(String),
    FontSize(Length),
    FontWeight(FontWeight),
    FontStyle(FontStyle),
    LineHeight(Length),
    TextAlign(TextAlign),
    TextDecoration(String),
    
    // Positioning
    Top(Length),
    Right(Length),
    Bottom(Length),
    Left(Length),
    ZIndex(i32),
    
    // Overflow
    OverflowX(Overflow),
    OverflowY(Overflow),
    
    // Visibility
    Visibility(bool),
    Opacity(f32),
    
    // Flexbox
    FlexDirection(String),
    FlexWrap(String),
    JustifyContent(String),
    AlignItems(String),
    AlignContent(String),
    FlexGrow(f32),
    FlexShrink(f32),
    FlexBasis(Length),
    
    // Other
    Cursor(String),
    PointerEvents(String),
    UserSelect(String),
    
    // Raw/unparsed
    Raw(String),
}

impl Property {
    pub fn new(name: &str, value: PropertyValue) -> Self {
        Self {
            name: name.to_string(),
            value,
            important: false,
        }
    }

    pub fn with_important(mut self) -> Self {
        self.important = true;
        self
    }
}

/// Parse a CSS property value
pub fn parse_property(name: &str, value: &str) -> Option<Property> {
    let value = value.trim();
    let (value, important) = if value.ends_with("!important") {
        (value.strip_suffix("!important")?.trim(), true)
    } else {
        (value, false)
    };

    let prop_value = match name {
        "display" => PropertyValue::Display(parse_display(value)?),
        "position" => PropertyValue::Position(parse_position(value)?),
        "float" => PropertyValue::Float(parse_float(value)?),
        
        "width" => PropertyValue::Width(parse_length(value)?),
        "height" => PropertyValue::Height(parse_length(value)?),
        "min-width" => PropertyValue::MinWidth(parse_length(value)?),
        "min-height" => PropertyValue::MinHeight(parse_length(value)?),
        "max-width" => PropertyValue::MaxWidth(parse_length(value)?),
        "max-height" => PropertyValue::MaxHeight(parse_length(value)?),
        
        "margin-top" => PropertyValue::MarginTop(parse_length(value)?),
        "margin-right" => PropertyValue::MarginRight(parse_length(value)?),
        "margin-bottom" => PropertyValue::MarginBottom(parse_length(value)?),
        "margin-left" => PropertyValue::MarginLeft(parse_length(value)?),
        
        "padding-top" => PropertyValue::PaddingTop(parse_length(value)?),
        "padding-right" => PropertyValue::PaddingRight(parse_length(value)?),
        "padding-bottom" => PropertyValue::PaddingBottom(parse_length(value)?),
        "padding-left" => PropertyValue::PaddingLeft(parse_length(value)?),
        
        "color" => PropertyValue::Color(parse_color(value)?),
        "background-color" => PropertyValue::BackgroundColor(parse_color(value)?),
        
        "font-size" => PropertyValue::FontSize(parse_length(value)?),
        "font-weight" => PropertyValue::FontWeight(parse_font_weight(value)?),
        "line-height" => PropertyValue::LineHeight(parse_length(value)?),
        "text-align" => PropertyValue::TextAlign(parse_text_align(value)?),
        
        "top" => PropertyValue::Top(parse_length(value)?),
        "right" => PropertyValue::Right(parse_length(value)?),
        "bottom" => PropertyValue::Bottom(parse_length(value)?),
        "left" => PropertyValue::Left(parse_length(value)?),
        "z-index" => PropertyValue::ZIndex(value.parse().ok()?),
        
        "overflow-x" => PropertyValue::OverflowX(parse_overflow(value)?),
        "overflow-y" => PropertyValue::OverflowY(parse_overflow(value)?),
        
        "opacity" => PropertyValue::Opacity(value.parse().ok()?),
        
        "flex-grow" => PropertyValue::FlexGrow(value.parse().ok()?),
        "flex-shrink" => PropertyValue::FlexShrink(value.parse().ok()?),
        
        _ => PropertyValue::Raw(value.to_string()),
    };

    let mut prop = Property::new(name, prop_value);
    if important {
        prop = prop.with_important();
    }
    Some(prop)
}

fn parse_length(value: &str) -> Option<Length> {
    if value == "0" {
        return Some(Length::Zero);
    }
    if value == "auto" {
        return Some(Length::Auto);
    }
    
    if let Some(v) = value.strip_suffix("px") {
        return Some(Length::Px(v.parse().ok()?));
    }
    if let Some(v) = value.strip_suffix("em") {
        return Some(Length::Em(v.parse().ok()?));
    }
    if let Some(v) = value.strip_suffix("rem") {
        return Some(Length::Rem(v.parse().ok()?));
    }
    if let Some(v) = value.strip_suffix('%') {
        return Some(Length::Percent(v.parse().ok()?));
    }
    if let Some(v) = value.strip_suffix("vw") {
        return Some(Length::Vw(v.parse().ok()?));
    }
    if let Some(v) = value.strip_suffix("vh") {
        return Some(Length::Vh(v.parse().ok()?));
    }
    
    // Try parsing as pixels without unit
    value.parse().ok().map(Length::Px)
}

fn parse_color(value: &str) -> Option<Color> {
    if value.starts_with('#') {
        return Color::from_hex(value);
    }
    if value.starts_with("rgb") {
        // Parse rgb() or rgba()
        let inner = value
            .strip_prefix("rgba(")
            .or_else(|| value.strip_prefix("rgb("))?
            .strip_suffix(')')?;
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            let r: u8 = parts[0].trim().parse().ok()?;
            let g: u8 = parts[1].trim().parse().ok()?;
            let b: u8 = parts[2].trim().parse().ok()?;
            let a: u8 = if parts.len() > 3 {
                (parts[3].trim().parse::<f32>().ok()? * 255.0) as u8
            } else {
                255
            };
            return Some(Color::rgba(r, g, b, a));
        }
    }
    Color::from_name(value)
}

fn parse_display(value: &str) -> Option<Display> {
    match value {
        "none" => Some(Display::None),
        "block" => Some(Display::Block),
        "inline" => Some(Display::Inline),
        "inline-block" => Some(Display::InlineBlock),
        "flex" => Some(Display::Flex),
        "grid" => Some(Display::Grid),
        "contents" => Some(Display::Contents),
        _ => None,
    }
}

fn parse_position(value: &str) -> Option<Position> {
    match value {
        "static" => Some(Position::Static),
        "relative" => Some(Position::Relative),
        "absolute" => Some(Position::Absolute),
        "fixed" => Some(Position::Fixed),
        "sticky" => Some(Position::Sticky),
        _ => None,
    }
}

fn parse_float(value: &str) -> Option<Float> {
    match value {
        "none" => Some(Float::None),
        "left" => Some(Float::Left),
        "right" => Some(Float::Right),
        _ => None,
    }
}

fn parse_overflow(value: &str) -> Option<Overflow> {
    match value {
        "visible" => Some(Overflow::Visible),
        "hidden" => Some(Overflow::Hidden),
        "scroll" => Some(Overflow::Scroll),
        "auto" => Some(Overflow::Auto),
        _ => None,
    }
}

fn parse_text_align(value: &str) -> Option<TextAlign> {
    match value {
        "left" => Some(TextAlign::Left),
        "right" => Some(TextAlign::Right),
        "center" => Some(TextAlign::Center),
        "justify" => Some(TextAlign::Justify),
        _ => None,
    }
}

fn parse_font_weight(value: &str) -> Option<FontWeight> {
    match value {
        "normal" => Some(FontWeight::NORMAL),
        "bold" => Some(FontWeight::BOLD),
        _ => value.parse().ok().map(FontWeight),
    }
}
