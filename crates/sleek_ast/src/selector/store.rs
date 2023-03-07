use std::mem::take;

use crate::HtmlTag;

use super::{
    parser::{Emit, Relation},
    Selector, SelectorError, SelectorPattern,
};

pub struct SelectorStore {
    pub(crate) selectors: Vec<Selector>,
    pub(crate) has_data: bool,
    pub(crate) cache: [String; 2],
}

impl SelectorStore {
    /// Create a new store.
    pub fn new() -> Self {
        SelectorStore {
            selectors: vec![Selector::new()],
            cache: [String::new(), String::new()],
            has_data: false,
        }
    }
    /// Returns a reference to the main selector in the store.
    pub fn host(&self) -> &Selector {
        &self.selectors[0]
    }
    /// Add a new selector.
    pub fn emit(&mut self, event: Emit) -> Result<(), SelectorError> {
        // Can only create a pattern if there is data in the cache.
        if !self.has_data {
            return Err(SelectorError::InvalidSelector);
        }

        let data = take(&mut self.cache[0]);

        // Confirm selector type.
        let pattern = match event {
            // Creates a tag pattern.
            Emit::Tag => SelectorPattern::Tag(HtmlTag::new(data)),
            // Creates an #id pattern.
            Emit::Id => {
                let would_be_pattern = SelectorPattern::Id(data);
                // Prevent id clashes.
                if self.host().patterns.contains(&would_be_pattern) {
                    Err(SelectorError::MultipleIds)?
                }
                would_be_pattern
            }
            // Creates a .class pattern.
            Emit::Class => SelectorPattern::Class(data),
            // Creates a * class pattern.
            Emit::Universal => SelectorPattern::Universal,
            // Creates an [attribute] pattern.
            Emit::Attribute => SelectorPattern::Attribute(
                data,
                if self.cache[1].is_empty() {
                    None
                } else {
                    Some(take(&mut self.cache[1]))
                },
            ),
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
        self.cache[0].push(ch);
    }
    pub fn collect_2(&mut self, ch: char) {
        if !self.has_data {
            self.has_data = true;
        }
        self.cache[1].push(ch);
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
