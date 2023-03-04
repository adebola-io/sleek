use std::mem::take;

use crate::HtmlTag;

use super::{
    parser::{Emit, Relation},
    Selector, SelectorError, SelectorPattern,
};

pub struct SelectorStore {
    pub selectors: Vec<Selector>,
    pub has_data: bool,
    cache: String,
}

impl SelectorStore {
    /// Create a new store.
    pub fn new() -> Self {
        SelectorStore {
            selectors: vec![Selector::new()],
            cache: String::new(),
            has_data: false,
        }
    }
    /// Returns a reference to the main selector in the store.
    pub fn host(&self) -> &Selector {
        &self.selectors[0]
    }
    /// Adds an attribute value to the host selector.
    pub fn emit_attribute_value(&mut self) -> Result<(), SelectorError> {
        if !self.has_data {
            return Err(SelectorError::InvalidSelector);
        }
        if let Some(SelectorPattern::Attribute(_, value)) = self.selectors[0].patterns.last_mut() {
            value.replace(take(&mut self.cache));
        }
        self.has_data = false;
        Ok(())
    }
    /// Add a new selector.
    pub fn emit(&mut self, event: Emit) -> Result<(), SelectorError> {
        // Can only create a pattern if there is data in the cache.
        if !self.has_data {
            return Err(SelectorError::InvalidSelector);
        }

        // Confirm selector type.
        let pattern = match event {
            // Creates a tag pattern.
            Emit::Tag => SelectorPattern::Tag(HtmlTag::new(take(&mut self.cache))),
            // Creates an #id pattern.
            Emit::Id => {
                let would_be_pattern = SelectorPattern::Id(take(&mut self.cache));
                // Prevent id clashes.
                if self.host().patterns.contains(&would_be_pattern) {
                    Err(SelectorError::MultipleIds)?
                }
                would_be_pattern
            }
            // Creates a .class pattern.
            Emit::Class => SelectorPattern::Class(take(&mut self.cache)),
            // Creates a * class pattern.
            Emit::Universal => SelectorPattern::Universal,
            // Creates an [attribute] pattern.
            Emit::AttribName => SelectorPattern::Attribute(take(&mut self.cache), None),
            // Creates a [attribute=value] pattern.
            _ => unreachable!(),
        };

        // Check previous selector for relation.
        match self.selectors[0].patterns.get_mut(0) {
            // Parsed selector is a descendant of previous.
            Some(SelectorPattern::Descendant([_, child])) => child.patterns.push(pattern),
            // Parsed selector is simple.
            _ => {
                self.selectors[0].patterns.push(pattern);
            }
        }
        self.has_data = false;
        Ok(())
    }
    /// Pushes a character to the store cache.
    pub fn collect(&mut self, ch: char) {
        if !self.has_data {
            self.has_data = true;
        }
        self.cache.push(ch);
    }
    /// Creates a new host selector with a relation to the current host selector.
    pub fn shift(&mut self, relation: Relation) {
        match relation {
            // A catch-all selector, e.g. "div span", a span at any level within a div.
            Relation::Descendant => {
                // Move current selector to dec
                let last = self.selectors.pop().unwrap();
                self.selectors.push(Selector::new());
                self.selectors[0]
                    .patterns
                    .push(SelectorPattern::Descendant([last, Selector::new()]));
            }
            // Direct child selector, e.g. "section > h1", a h1 which is an immediate child of section.
            Relation::Child => {
                let last = self.selectors.pop().unwrap();
                self.selectors.push(Selector::new());
                self.selectors[0]
                    .patterns
                    .push(SelectorPattern::Child([last, Selector::new()]));
            }
            Relation::AdjacentSibling => todo!(),
            Relation::GeneralSibling => todo!(),
            Relation::Group => todo!(),
        }
    }
}
