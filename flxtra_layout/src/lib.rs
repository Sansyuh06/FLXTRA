//! CSS Layout Engine
//! 
//! Responsible for:
//! - Implementing the CSS box model (block, inline, inline-block)
//! - Calculating layout (width, height, position, margin, padding)
//! - Handling floats, positioning, and basic flex/grid
//!
//! Produces: Layout tree with computed geometry for rendering

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub padding: Spacing,
    pub margin: Spacing,
    pub border: Spacing,
}

#[derive(Debug, Clone)]
pub struct Spacing {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Spacing {
    pub fn zero() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_layout(
        width: f64,
        height: f64,
        properties: &HashMap<String, String>,
    ) -> LayoutBox {
        let mut layout = LayoutBox {
            x: 0.0,
            y: 0.0,
            width,
            height,
            padding: Spacing::zero(),
            margin: Spacing::zero(),
            border: Spacing::zero(),
        };

        // Parse padding
        if let Some(padding_str) = properties.get("padding") {
            layout.padding = parse_spacing(padding_str);
        }

        // Parse margin
        if let Some(margin_str) = properties.get("margin") {
            layout.margin = parse_spacing(margin_str);
        }

        // Parse width/height
        if let Some(w_str) = properties.get("width") {
            layout.width = parse_size(w_str, width);
        }
        if let Some(h_str) = properties.get("height") {
            layout.height = parse_size(h_str, height);
        }

        layout
    }
}

fn parse_spacing(spacing_str: &str) -> Spacing {
    let parts: Vec<&str> = spacing_str.split_whitespace().collect();
    match parts.len() {
        1 => {
            let val = parse_value(parts[0]);
            Spacing {
                top: val,
                right: val,
                bottom: val,
                left: val,
            }
        }
        2 => {
            let vertical = parse_value(parts[0]);
            let horizontal = parse_value(parts[1]);
            Spacing {
                top: vertical,
                right: horizontal,
                bottom: vertical,
                left: horizontal,
            }
        }
        4 => Spacing {
            top: parse_value(parts[0]),
            right: parse_value(parts[1]),
            bottom: parse_value(parts[2]),
            left: parse_value(parts[3]),
        },
        _ => Spacing::zero(),
    }
}

fn parse_size(size_str: &str, parent: f64) -> f64 {
    if size_str.ends_with('%') {
        let percent = size_str.trim_end_matches('%').parse::<f64>().unwrap_or(100.0);
        parent * (percent / 100.0)
    } else {
        parse_value(size_str)
    }
}

fn parse_value(val_str: &str) -> f64 {
    val_str
        .trim_end_matches(|c: char| !c.is_numeric() && c != '.')
        .parse::<f64>()
        .unwrap_or(0.0)
}
