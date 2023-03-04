use sleek_utils::Node;

use super::{ElementRef, HtmlTag};

/// Trait for query selection and traversal using class names, ids and tags.
pub trait Query: Node<ElementRef> {
    /// Traverse tree and find the first element that matches a selector, if it exists.
    fn query_selector(&self, selector: &str) -> Option<ElementRef> {
        let mut selector_match = None;
        for reference in self.children() {
            if reference.matches(selector) {
                selector_match = Some(reference.clone());
                break;
            } else {
                selector_match = reference.query_selector(selector);
                if selector_match.is_some() {
                    break;
                }
            }
        }
        selector_match
    }
    /// Traverse tree and find all the elements that matches a selector.
    fn query_selector_all(&self, selector: &str) -> Vec<ElementRef> {
        let mut matches = vec![];
        for reference in self.children() {
            if reference.matches(selector) {
                matches.push(reference.clone())
            } else {
                matches.append(&mut reference.query_selector_all(selector));
            }
        }
        matches
    }
    fn get_elements_by_class_name(&self, class_name: &str) -> Vec<ElementRef> {
        let mut matches = vec![];
        for child in self.children() {
            let child_class_name = child.class_name();
            let list = child_class_name.split(' ').collect::<Vec<_>>();
            if list.contains(&class_name) {
                matches.push(child.clone())
            } else {
                matches.append(&mut child.get_elements_by_class_name(class_name));
            }
        }
        matches
    }
    fn get_element_by_id(&self, id: &str) -> Option<ElementRef> {
        let mut id_match = None;
        for reference in self.children() {
            if let Some(_id) = reference.id() && _id == id {
                id_match = Some(reference.clone());
                break;
            } else {
                id_match = reference.get_element_by_id(id);
                if id_match.is_some() {
                    break;
                }
            }
        }
        id_match
    }
    fn get_elements_by_tag_name(&self, tag: &HtmlTag) -> Vec<ElementRef> {
        let mut matches = vec![];
        for child in self.children() {
            if &child.element.borrow().name == tag {
                matches.push(child.clone())
            } else {
                matches.append(&mut child.get_elements_by_tag_name(tag));
            }
        }
        matches
    }
}
