//! CSS Selector Map for fast lookup
//!
//! Optimizes selector matching by indexing selectors by their rightmost key

use crate::stylesheet::{Selector, SelectorPart, StyleDeclaration};
use std::collections::HashMap;

/// Optimized selector map for fast element matching
#[derive(Debug, Default)]
pub struct SelectorMap {
    /// Selectors indexed by ID
    by_id: HashMap<String, Vec<(usize, Selector)>>,
    /// Selectors indexed by class
    by_class: HashMap<String, Vec<(usize, Selector)>>,
    /// Selectors indexed by tag name
    by_tag: HashMap<String, Vec<(usize, Selector)>>,
    /// Universal selectors (must check all)
    universal: Vec<(usize, Selector)>,
    /// Total number of selectors
    count: usize,
}

impl SelectorMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build selector map from style declarations
    pub fn from_declarations(declarations: &[StyleDeclaration]) -> Self {
        let mut map = Self::new();
        
        for (index, decl) in declarations.iter().enumerate() {
            for selector in &decl.selectors {
                map.add_selector(index, selector.clone());
            }
        }
        
        map
    }

    /// Add a selector to the map
    pub fn add_selector(&mut self, index: usize, selector: Selector) {
        // Index by the rightmost significant part (most specific first)
        let mut indexed = false;
        
        for part in selector.parts.iter().rev() {
            match part {
                SelectorPart::Id(id) => {
                    self.by_id
                        .entry(id.to_lowercase())
                        .or_default()
                        .push((index, selector.clone()));
                    indexed = true;
                    break;
                }
                SelectorPart::Class(class) => {
                    self.by_class
                        .entry(class.to_lowercase())
                        .or_default()
                        .push((index, selector.clone()));
                    indexed = true;
                    break;
                }
                SelectorPart::Tag(tag) => {
                    self.by_tag
                        .entry(tag.to_lowercase())
                        .or_default()
                        .push((index, selector.clone()));
                    indexed = true;
                    break;
                }
                _ => continue,
            }
        }
        
        if !indexed {
            // Universal or complex selector
            self.universal.push((index, selector));
        }
        
        self.count += 1;
    }

    /// Get candidate selectors for an element
    /// Returns (rule_index, selector) pairs
    pub fn get_candidates(
        &self,
        tag: &str,
        id: Option<&str>,
        classes: &[&str],
    ) -> Vec<&(usize, Selector)> {
        let mut candidates = Vec::new();
        let tag_lower = tag.to_lowercase();
        
        // Add candidates from most specific to least specific
        if let Some(i) = id {
            if let Some(v) = self.by_id.get(&i.to_lowercase()) {
                candidates.extend(v.iter());
            }
        }
        
        for c in classes {
            if let Some(v) = self.by_class.get(&c.to_lowercase()) {
                candidates.extend(v.iter());
            }
        }
        
        if let Some(v) = self.by_tag.get(&tag_lower) {
            candidates.extend(v.iter());
        }
        
        candidates.extend(self.universal.iter());
        
        candidates
    }

    /// Number of indexed selectors
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_map() {
        let mut map = SelectorMap::new();
        map.add_selector(0, Selector::parse("#header"));
        map.add_selector(1, Selector::parse(".container"));
        map.add_selector(2, Selector::parse("div"));
        
        let candidates: Vec<_> = map.get_candidates("div", Some("header"), &["container"]).collect();
        assert!(candidates.len() >= 3); // Should find all three
    }
}
