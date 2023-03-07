use sleek_utils::Node;

use super::{ElementRef, HtmlTag};

/// This trait provides functionality for query selection for element trees and element themselves. It allows traversal using selectors, class names, ids and tags.
pub trait Query<'a>: Node<'a, ElementRef> {
    /// Traverse tree and find the first element that matches a selector, if it exists.
    fn query_selector(&'a self, selector: &str) -> Option<ElementRef> {
        for reference in self.children() {
            if reference.matches(selector) {
                return Some(reference.clone());
            }
            let selector_match = reference.query_selector(selector);
            if selector_match.is_some() {
                return selector_match;
            }
        }
        None
    }
    /// Traverse tree and find all the elements that matches a selector.
    fn query_selector_all(&'a self, selector: &str) -> Vec<&ElementRef> {
        let mut matches = vec![];
        for reference in self.children() {
            if reference.matches(selector) {
                matches.push(reference);
            }
            if reference.has_children() {
                matches.append(&mut reference.query_selector_all(selector));
            }
        }
        matches
    }
    /// Traverse element or tree and return all elements that have a particular class.
    fn get_elements_by_class_name(&'a self, class_name: &str) -> Vec<&ElementRef> {
        let mut matches = vec![];
        for child in self.children() {
            let child_class_name = child.class_name();
            let list = child_class_name.split_whitespace();

            for child_class_list_item in list {
                if child_class_list_item == class_name {
                    matches.push(child);
                    break;
                }
            }

            if child.has_children() {
                matches.append(&mut child.get_elements_by_class_name(class_name));
            }
        }
        matches
    }
    /// Return the first element in the tree/subtree that has a specified id, if it exists.
    fn get_element_by_id(&'a self, id: &str) -> Option<&ElementRef> {
        for reference in self.children() {
            if let Some(_id) = reference.id() && _id == id {
                return Some(reference);
            }
            let id_match = reference.get_element_by_id(id);
            if id_match.is_some() {
                return id_match;
            }
        }
        None
    }
    /// Return all elements in the tree/subtree that have a specified tag name.
    fn get_elements_by_tag_name(&'a self, tag: &HtmlTag) -> Vec<&ElementRef> {
        let mut matches = vec![];
        for child in self.children() {
            if child.tag_name() == tag {
                matches.push(child);
            }
            if child.has_children() {
                matches.append(&mut child.get_elements_by_tag_name(tag));
            }
        }
        matches
    }
}
