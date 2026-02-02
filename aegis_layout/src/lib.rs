//! Aegis Layout Engine
//! CSS box model layout with block/inline/flex support

pub mod box_model;
pub mod tree;
pub mod engine;

pub use box_model::{LayoutBox, BoxDimensions, Rect};
pub use tree::LayoutTree;
pub use engine::LayoutEngine;
