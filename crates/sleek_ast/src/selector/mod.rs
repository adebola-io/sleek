mod parser;
mod pattern;
mod store;

pub use parser::{parse_selector, SelectorError};
pub use pattern::SelectorPattern;
use sleek_utils::Node;
pub use store::SelectorStore;

use crate::ElementRef;

#[derive(PartialEq, Debug)]
pub struct Selector {
    patterns: Vec<SelectorPattern>,
}

impl Selector {
    pub fn new() -> Self {
        Selector { patterns: vec![] }
    }
    pub fn compare(&self, element_ref: &ElementRef) -> bool {
        for pattern in &self.patterns {
            match pattern {
                SelectorPattern::Universal => {
                    // All elements match.
                }
                SelectorPattern::Tag(tag) if element_ref.tag_name() != tag => {
                    return false;
                }
                SelectorPattern::Class(class_name)
                    if !element_ref.class_list().contains(class_name) =>
                {
                    return false;
                }
                SelectorPattern::Id(id) => match element_ref.id() {
                    Some(element_id) => {
                        if element_id != id {
                            return false;
                        }
                    }
                    None => return false,
                },
                SelectorPattern::Attribute(key, value_opt) => {
                    match element_ref.get_attribute(key) {
                        None => return false,
                        Some(element_value) => {
                            if let Some(s_value) = value_opt {
                                if s_value != element_value {
                                    return false;
                                }
                            }
                        }
                    }
                }
                SelectorPattern::Descendant(relation) => {
                    if !relation[1].compare(element_ref) {
                        return false;
                    }
                    let mut parent = element_ref.parent();
                    loop {
                        if let Some(parent_ref) = parent {
                            if relation[0].compare(&parent_ref) {
                                break;
                            } else {
                                // If the immediate parent is not found, go up one level.
                                parent = parent_ref.parent();
                            }
                        } else {
                            return false;
                        }
                    }
                }
                SelectorPattern::Child(relation) => {
                    if !relation[1].compare(element_ref) {
                        return false;
                    }
                    match element_ref.parent() {
                        Some(parent_ref) => {
                            if !relation[0].compare(&parent_ref) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                // SelectorPattern::Parent(relation) => {
                //     if !relation[0].compare(element_ref) {
                //         return false;
                //     }
                //     let mut match_found = false;
                //     for child in element_ref.children() {
                //         if relation[1].compare(&child) {
                //             match_found = true;
                //             break;
                //         }
                //     }
                //     if !match_found {
                //         return false;
                //     }
                // }
                SelectorPattern::AdjacentSibling(relation) => {
                    if !relation[1].compare(element_ref) {
                        return false;
                    }
                    match element_ref.parent() {
                        None => return false,
                        Some(parent_ref) => {
                            let mut index = parent_ref.get_index_of(element_ref).unwrap();
                            if index == 0 {
                                return false;
                            }
                            index -= 1;
                            // Find the nearest element before.
                            let mut adjacent_ref = &parent_ref.element().child_nodes[index];
                            while !(adjacent_ref.is_element()) && index > 0 {
                                adjacent_ref = &parent_ref.element().child_nodes[index - 1];
                            }
                            if !(adjacent_ref.is_element()
                                && relation[0].compare(&adjacent_ref.as_element_ref().unwrap()))
                            {
                                return false;
                            }
                        }
                    }
                }
                SelectorPattern::GeneralSibling(relation) => {
                    if !relation[1].compare(element_ref) {
                        return false;
                    }
                    match element_ref.parent() {
                        None => return false,
                        Some(parent_ref) => {
                            let mut match_found = false;
                            for sibling_ref in parent_ref.children() {
                                if sibling_ref != element_ref && relation[0].compare(&sibling_ref) {
                                    match_found = true;
                                    break;
                                }
                            }
                            if !match_found {
                                return false;
                            }
                        }
                    }
                }
                SelectorPattern::Group(patterns) => {
                    let mut match_found = false;
                    for pattern in patterns {
                        if pattern.compare(element_ref) {
                            match_found = true;
                            break;
                        }
                    }
                    if !match_found {
                        return false;
                    }
                }
                SelectorPattern::PseudoClass(_) => todo!(),
                _ => {}
            }
        }
        true
    }
}
