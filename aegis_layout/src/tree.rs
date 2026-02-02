//! Layout tree construction
use crate::box_model::{BoxType, LayoutBox};
use aegis_html::dom::{Element, Node, NodeData};
use std::sync::Arc;

pub struct LayoutTree;

impl LayoutTree {
    pub fn build(root: &Arc<Node>) -> LayoutBox {
        Self::build_box(root)
    }

    fn build_box(node: &Arc<Node>) -> LayoutBox {
        match &node.data {
            NodeData::Element(elem) => {
                let box_type = Self::box_type_for_tag(&elem.tag_name);
                if box_type == BoxType::Anonymous { return LayoutBox::new(BoxType::Anonymous); }
                
                let mut layout_box = LayoutBox::new(box_type);
                layout_box.node_id = Some(node.id.0);
                
                for child in node.children.read().iter() {
                    let child_box = Self::build_box(child);
                    if child_box.box_type != BoxType::Anonymous || !child_box.children.is_empty() {
                        layout_box.add_child(child_box);
                    }
                }
                layout_box
            }
            NodeData::Text(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() { LayoutBox::new(BoxType::Anonymous) }
                else { LayoutBox::text(trimmed.to_string()) }
            }
            _ => LayoutBox::new(BoxType::Anonymous),
        }
    }

    fn box_type_for_tag(tag: &str) -> BoxType {
        match tag.to_lowercase().as_str() {
            "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "header" | 
            "footer" | "main" | "section" | "article" | "nav" | "aside" |
            "ul" | "ol" | "li" | "table" | "form" | "blockquote" | "pre" => BoxType::Block,
            "span" | "a" | "strong" | "em" | "b" | "i" | "code" | "label" => BoxType::Inline,
            "img" | "button" | "input" | "select" | "textarea" => BoxType::InlineBlock,
            "script" | "style" | "meta" | "link" | "head" | "title" => BoxType::Anonymous,
            _ => BoxType::Block,
        }
    }
}
