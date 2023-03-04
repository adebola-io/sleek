use std::{cell::RefCell, collections::HashMap, rc::Rc};

use sleek_utils::{MutableCountRef, Node};

use crate::parse_selector;

use super::{ElementSpan, HtmlEventListener, HtmlNode, HtmlTag, Query};

#[derive(Debug)]
pub struct Element {
    pub name: HtmlTag,
    pub class_list: RefCell<Vec<String>>,
    pub attributes: RefCell<HashMap<String, String>>,
    _listeners: Vec<HtmlEventListener>,
    pub location: ElementSpan,
    pub child_nodes: Vec<HtmlNode>,
    pub parent_ref: Option<ElementRef>,
}

impl Element {
    pub fn new(name: HtmlTag) -> Self {
        Element {
            name,
            attributes: RefCell::new(HashMap::new()),
            class_list: RefCell::new(vec![]),
            _listeners: vec![],
            location: ElementSpan::empty(),
            child_nodes: vec![],
            parent_ref: None,
        }
    }
}

/// [ElementRef] is the base struct for an element in a document.
/// It implements common methods for document operations, such as [class_list](#172), [append](Node::append) and [query_selector](Query::query_selector).
/// The struct is a wrapper referencing an [Element] object, which allows it to be:
/// - replicated easily without copying the underlying element.
/// - arranged in a tree structure without breaking Rust's rules.
#[derive(Debug, Clone)]
pub struct ElementRef {
    pub element: MutableCountRef<Element>,
}

impl ElementRef {
    pub fn new(tag_name: &str) -> Self {
        ElementRef {
            element: Rc::new(RefCell::new(Element::new(HtmlTag::new(
                tag_name.to_string(),
            )))),
        }
    }
    pub fn from(name: HtmlTag) -> Self {
        ElementRef {
            element: Rc::new(RefCell::new(Element::new(name))),
        }
    }
    pub fn over(element: Element) -> Self {
        ElementRef {
            element: Rc::new(RefCell::new(element)),
        }
    }
}

impl ElementRef {
    /// Returns the tagname of the element.
    pub fn tag_name(&self) -> String {
        self.element.borrow().name.to_string()
    }
    /// Returns the id of an element if it exists.
    pub fn id(&self) -> Option<String> {
        self.get_attribute("id")
    }
    pub fn set_inner_html(&mut self) {
        todo!()
    }
    /// Check if element matches a CSS style selector.
    pub fn matches(&self, selector: &str) -> bool {
        match parse_selector(selector) {
            Ok(selector_store) => selector_store.host().compare(self),
            Err(e) => panic!("{:?}", e),
        }
    }
    /// Return a string with the inner text of the node.
    pub fn get_text_content(&self) -> String {
        let mut text_content = String::new();
        for node in &self.element.borrow().child_nodes {
            match node {
                HtmlNode::Text(text_ref) => {
                    text_content.push_str(text_ref.text.borrow().content.as_str())
                }
                HtmlNode::Element(element_ref) => {
                    text_content.push_str(element_ref.get_text_content().as_str())
                }
                _ => {}
            }
        }
        text_content
    }
    /// Retrieve an attribute of the element.
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        if let Some(s) = self.element.borrow().attributes.borrow().get(name) {
            Some(s.clone())
        } else {
            None
        }
    }
    /// Set an attribute on the element.
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.element
            .borrow()
            .attributes
            .borrow_mut()
            .insert(name.to_string(), value.to_string());

        if name == "class" {
            self.update_class_list();
        }
    }
    /// Remove an attribute from the element.
    pub fn remove_attribute(&self, qualified_name: &str) {
        self.element
            .borrow()
            .attributes
            .borrow_mut()
            .remove(qualified_name);
    }
    /// Get the index of a child node.
    pub fn get_index_of(&self, child: &ElementRef) -> Option<usize> {
        self.element
            .borrow()
            .child_nodes
            .iter()
            .enumerate()
            .find(|enum_item| {
                if let HtmlNode::Element(e) = enum_item.1 {
                    if e == child {
                        return true;
                    }
                }
                false
            })
            .map(|tuple| tuple.0)
    }
}

impl ElementRef {
    /// Returns the class name of the element.
    pub fn class_name(&self) -> String {
        self.class_list().join(" ")
    }
    /// Adds a new class to the class list of the element.
    pub fn add_class(&mut self, class_name: &str) {
        self.element
            .borrow()
            .class_list
            .borrow_mut()
            .push(class_name.to_string());

        self.element
            .borrow()
            .attributes
            .borrow_mut()
            .insert("class".to_string(), self.class_name() + " " + class_name);
    }
    /// Removes a class from the element class list if it exists.
    pub fn remove_class(&mut self, class_name: &str) {
        self.element
            .borrow()
            .class_list
            .borrow_mut()
            .retain(|class| class != class_name);
    }
    /// Returns the class list of the element.
    pub fn class_list(&self) -> Vec<String> {
        self.element.borrow().class_list.borrow().clone()
    }
    /// Updates the class list. Useful whenever the class attribute changes directly.
    fn update_class_list(&self) {
        let element = &self.element.borrow();
        let mut list = element.class_list.borrow_mut();
        list.clear();
        if let Some(value) = self.get_attribute("class") {
            value
                .split(' ')
                .for_each(|token| list.push(token.to_owned()));
        }
    }
}

impl Node<ElementRef> for ElementRef {
    fn parent(&self) -> Option<ElementRef> {
        self.element.borrow().parent_ref.clone()
    }

    fn children(&self) -> Vec<ElementRef> {
        let mut children = vec![];
        for node in &self.element.borrow().child_nodes {
            if let Some(element_ref) = node.as_element_ref() {
                children.push(element_ref);
            }
        }
        children
    }

    fn append(&mut self, child: &ElementRef) {
        child.element.borrow_mut().parent_ref = Some(self.clone());
        self.element
            .borrow_mut()
            .child_nodes
            .push(HtmlNode::Element(child.clone()));
    }

    fn prepend(&mut self, child: &ElementRef) {
        child.element.borrow_mut().parent_ref = Some(self.clone());
        self.element
            .borrow_mut()
            .child_nodes
            .insert(0, HtmlNode::Element(child.clone()));
    }

    fn remove(&mut self, node: &ElementRef) {
        self.element
            .borrow_mut()
            .child_nodes
            .retain(|n| match &n.as_element_ref() {
                Some(r) => r != node,
                None => true,
            });
        node.element.borrow_mut().parent_ref = None;
    }

    fn after(&mut self, node: &ElementRef) {
        if let Some(value) = &self.element.borrow().parent_ref {
            let index = value.get_index_of(self).unwrap();
            value
                .element
                .borrow_mut()
                .child_nodes
                .insert(index + 1, HtmlNode::Element(node.clone()));
        }
    }
}

impl Query for ElementRef {}

impl PartialEq for ElementRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.element, &other.element)
    }
}
