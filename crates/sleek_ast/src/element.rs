use std::{cell::RefCell, collections::HashMap, rc::Rc};

use sleek_utils::{MutableCountRef, Node};

use crate::{parse_selector, AttributeQuoteType, HtmlAttribute, Span, TextRef};

use super::{ElementSpan, HtmlEventListener, HtmlNode, HtmlTag, Query};

pub struct Element {
    pub name: HtmlTag,
    pub class_list: RefCell<Vec<String>>,
    pub attributes: RefCell<HashMap<String, AttributeData>>,
    _listeners: Vec<HtmlEventListener>,
    pub location: ElementSpan,
    pub child_nodes: Vec<HtmlNode>,
    pub parent_ref: Option<ElementRef>,
}

#[derive(Debug)]
pub struct AttributeData {
    pub data: Option<String>,
    pub _quote_type: AttributeQuoteType,
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
    /// Manually Initialize the element to some set values.
    fn init(&mut self, attributes: Vec<HtmlAttribute>, start_tag_span: Span) {
        for attribute in attributes {
            self.attributes.borrow_mut().insert(
                attribute.key,
                AttributeData {
                    data: attribute.value,
                    _quote_type: attribute.quote_type,
                },
            );
        }

        self.location.open_tag = start_tag_span;
    }
}

/// ElementRef is the base struct for an element in a document.
/// It implements common methods for document operations, such as [class_list](#172), [append](Node::append) and [query_selector](Query::query_selector).
/// The struct is a wrapper referencing an [Element] object, which allows it to be:
/// - replicated easily without copying the underlying element.
/// - arranged in a tree structure without breaking Rust's rules (too much).
#[derive(Clone)]
pub struct ElementRef {
    pub __element: MutableCountRef<Element>,
}

// Exclude references from being displayed.
impl std::fmt::Debug for ElementRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let element = self.__element.borrow();
        let class_list = element.class_list.borrow();
        let attributes = element.attributes.borrow();
        let mut name = format!("{}", element.name).to_ascii_uppercase();
        name.push_str("Element");
        f.debug_struct(name.as_str())
            .field("attributes", &attributes)
            .field("location", &element.location)
            .field("class_list", &class_list)
            .field("children", &element.child_nodes)
            .finish()
    }
}

impl ElementRef {
    pub fn new(tag_name: &str) -> Self {
        ElementRef {
            __element: Rc::new(RefCell::new(Element::new(HtmlTag::new(
                tag_name.to_string(),
            )))),
        }
    }
    pub fn from(name: HtmlTag) -> Self {
        ElementRef {
            __element: Rc::new(RefCell::new(Element::new(name))),
        }
    }
    pub fn over(element: Element) -> Self {
        ElementRef {
            __element: Rc::new(RefCell::new(element)),
        }
    }
    pub fn init(name: HtmlTag, attributes: Vec<HtmlAttribute>, start_tag_span: Span) -> Self {
        let element_ref = Self::from(name);
        element_ref
            .__element
            .borrow_mut()
            .init(attributes, start_tag_span);
        element_ref.update_class_list();
        element_ref
    }
}

impl ElementRef {
    /// Returns the tagname of the element.
    pub fn tag_name(&self) -> String {
        self.__element.borrow().name.to_string()
    }
    /// Return the ending of the element in its original document.
    pub fn get_end(&self) -> [usize; 2] {
        let element = self.__element.borrow();
        if let Some(span) = &element.location.close_tag {
            span.end
        } else {
            element.location.open_tag.end
        }
    }

    /// Returns the id of an element if it exists.
    pub fn id(&self) -> Option<String> {
        self.get_attribute("id")
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
        for node in &self.__element.borrow().child_nodes {
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
        if let Some(s) = self.__element.borrow().attributes.borrow().get(name) {
            s.data.clone()
        } else {
            None
        }
    }
    /// Set an attribute on the element.
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.__element.borrow().attributes.borrow_mut().insert(
            name.to_string(),
            AttributeData {
                data: Some(value.to_string()),
                _quote_type: AttributeQuoteType::Double,
            },
        );

        if name == "class" {
            self.update_class_list();
        }
    }
    /// Remove an attribute from the element.
    pub fn remove_attribute(&self, qualified_name: &str) {
        self.__element
            .borrow()
            .attributes
            .borrow_mut()
            .remove(qualified_name);
    }
    /// Get the index of a child node.
    pub fn get_index_of(&self, child: &ElementRef) -> Option<usize> {
        self.__element
            .borrow()
            .child_nodes
            .iter()
            .enumerate()
            .find(|enum_item| match enum_item.1 {
                HtmlNode::Element(e) if e == child => true,
                _ => false,
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
        self.__element
            .borrow()
            .class_list
            .borrow_mut()
            .push(class_name.to_string());

        self.__element.borrow().attributes.borrow_mut().insert(
            "class".to_string(),
            AttributeData {
                data: Some(self.class_name() + " " + class_name),
                _quote_type: AttributeQuoteType::Double,
            },
        );
    }
    /// Removes a class from the element class list if it exists.
    pub fn remove_class(&mut self, class_name: &str) {
        self.__element
            .borrow()
            .class_list
            .borrow_mut()
            .retain(|class| class != class_name);
    }
    /// Returns the class list of the element.
    pub fn class_list(&self) -> Vec<String> {
        self.__element.borrow().class_list.borrow().clone()
    }
    /// Updates the class list. Useful whenever the class attribute changes directly.
    fn update_class_list(&self) {
        let element = &self.__element.borrow();
        let mut list = element.class_list.borrow_mut();
        list.clear();
        if let Some(value) = self.get_attribute("class") {
            value
                .split(' ')
                .for_each(|token| list.push(token.to_owned()));
        }
    }
    /// Appends a text node to the element.
    pub fn append_text(&mut self, text_ref: TextRef) {
        self.__element
            .borrow_mut()
            .child_nodes
            .push(HtmlNode::Text(text_ref));
    }
}

impl<'a> Node<'a, ElementRef> for ElementRef {
    fn parent(&self) -> Option<ElementRef> {
        self.__element.borrow().parent_ref.clone()
    }

    fn append(&mut self, child: &ElementRef) {
        child.__element.borrow_mut().parent_ref = Some(self.clone());
        self.__element
            .borrow_mut()
            .child_nodes
            .push(HtmlNode::Element(child.clone()));
    }

    fn prepend(&mut self, child: &ElementRef) {
        child.__element.borrow_mut().parent_ref = Some(self.clone());
        self.__element
            .borrow_mut()
            .child_nodes
            .insert(0, HtmlNode::Element(child.clone()));
    }

    fn has_children(&self) -> bool {
        self.__element.borrow().child_nodes.len() > 0
    }

    fn remove(&mut self, node: &ElementRef) {
        self.__element
            .borrow_mut()
            .child_nodes
            .retain(|n| match &n.as_element_ref() {
                Some(r) => r != node,
                None => true,
            });
        node.__element.borrow_mut().parent_ref = None;
    }

    fn after(&mut self, node: &ElementRef) {
        if let Some(value) = &self.__element.borrow().parent_ref {
            let index = value.get_index_of(self).unwrap();
            value
                .__element
                .borrow_mut()
                .child_nodes
                .insert(index + 1, HtmlNode::Element(node.clone()));
        }
    }

    fn children(&self) -> impl Iterator<Item = &'a ElementRef> {
        unsafe {
            (*self.__element.as_ptr())
                .child_nodes
                .iter()
                .filter(|node| node.is_element())
                .map(|node| match node {
                    HtmlNode::Element(e) => e,
                    _ => todo!(),
                })
        }
    }
}

impl Query<'_> for ElementRef {}

impl PartialEq for ElementRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.__element, &other.__element)
    }
}
