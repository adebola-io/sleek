use std::collections::HashMap;

use sleek_utils::Node;

use crate::{parse_selector, AttributeQuoteType, HtmlAttribute, HtmlTextNode, Span};

use super::{ElementSpan, HtmlEventListener, HtmlNode, HtmlTag, Query};

pub struct Element {
    pub name: HtmlTag,
    pub class_list: Vec<String>,
    pub refs: usize,
    pub attributes: HashMap<String, AttributeData>,
    _listeners: Vec<HtmlEventListener>,
    pub location: ElementSpan,
    pub child_nodes: Vec<HtmlNode>,
    pub __parent: Option<*mut Self>,
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
            attributes: HashMap::new(),
            refs: 0,
            class_list: vec![],
            _listeners: vec![],
            location: ElementSpan::empty(),
            child_nodes: vec![],
            __parent: None,
        }
    }
    /// Manually Initialize the element to some set values.
    fn init(&mut self, attributes: Vec<HtmlAttribute>, start_tag_span: Span) {
        for attribute in attributes {
            self.attributes.insert(
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
///
/// This struct should always be used in place of [Element] to prevent double-free.
pub struct ElementRef {
    __element: *mut Element,
}

// Exclude references from being displayed.
impl std::fmt::Debug for ElementRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            let element = &*self.__element;
            let element_name = element.name.to_string();
            let mut name_chars = element_name.chars();
            let formatted_name = format!(
                "{}{}Element",
                name_chars.next().unwrap().to_ascii_uppercase(),
                name_chars.collect::<String>()
            );
            f.debug_struct(formatted_name.as_str())
                .field("attributes", &element.attributes)
                .field("location", &element.location)
                .field("class_list", &element.class_list)
                .field("children", &element.child_nodes)
                .finish()
        }
    }
}

impl ElementRef {
    pub fn new(tag_name: &str) -> Self {
        Self::from(HtmlTag::new(tag_name.to_string()))
    }
    pub fn from(name: HtmlTag) -> Self {
        let mut element = Element::new(name);
        element.refs += 1;
        let __element = Box::into_raw(Box::new(element));
        ElementRef { __element }
    }
    pub fn over(__element: *mut Element) -> Self {
        unsafe {
            (*__element).refs += 1;
            ElementRef { __element }
        }
    }
    pub fn init(name: HtmlTag, attributes: Vec<HtmlAttribute>, start_tag_span: Span) -> Self {
        unsafe {
            let element_ref = Self::from(name);
            (*element_ref.__element).init(attributes, start_tag_span);
            element_ref.update_class_list();
            element_ref
        }
    }
    pub fn element(&self) -> &mut Element {
        unsafe { &mut *self.__element }
    }
}

impl ElementRef {
    /// Returns the tagname of the element.
    pub fn tag_name(&self) -> &HtmlTag {
        &self.element().name
    }
    /// Return the ending of the element in its original document.
    pub fn get_end(&self) -> [usize; 2] {
        if let Some(span) = &self.element().location.close_tag {
            span.end
        } else {
            self.element().location.open_tag.end
        }
    }

    /// Returns the id of an element if it exists.
    pub fn id(&self) -> &Option<String> {
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
        for node in &self.element().child_nodes {
            match node {
                HtmlNode::Text(text_node) => text_content.push_str(text_node.content.as_str()),
                HtmlNode::Element(element_ref) => {
                    text_content.push_str(element_ref.get_text_content().as_str())
                }
                _ => {}
            }
        }
        text_content
    }
    /// Retrieve an attribute of the element.
    pub fn get_attribute(&self, name: &str) -> &Option<String> {
        if let Some(s) = self.element().attributes.get(name) {
            &s.data
        } else {
            &None
        }
    }
    /// Set an attribute on the element.
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.element().attributes.insert(
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
        self.element().attributes.remove(qualified_name);
    }
    /// Get the index of a child node.
    pub fn get_index_of(&self, child: &ElementRef) -> Option<usize> {
        self.element()
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
        let element = self.element();
        element.class_list.push(class_name.to_string());

        element.attributes.insert(
            "class".to_string(),
            AttributeData {
                data: Some(self.class_name() + " " + class_name),
                _quote_type: AttributeQuoteType::Double,
            },
        );
    }
    /// Removes a class from the element class list if it exists.
    pub fn remove_class(&mut self, class_name: &str) {
        self.element()
            .class_list
            .retain(|class| class != class_name);
    }
    /// Returns the class list of the element.
    pub fn class_list(&self) -> &Vec<String> {
        &self.element().class_list
    }
    /// Updates the class list. Useful whenever the class attribute changes directly.
    fn update_class_list(&self) {
        let list = &mut self.element().class_list;
        list.clear();
        if let Some(value) = self.get_attribute("class") {
            value
                .split(' ')
                .for_each(|token| list.push(token.to_owned()));
        }
    }
    /// Appends a text node to the element.
    pub fn append_text(&mut self, text_node: HtmlTextNode) {
        self.element().child_nodes.push(HtmlNode::Text(text_node));
    }
}

impl<'a> Node<'a, ElementRef> for ElementRef {
    fn parent(&self) -> Option<ElementRef> {
        unsafe {
            match self.element().__parent {
                Some(pointer) => Some(Self::over(&mut *pointer)),
                None => None,
            }
        }
    }

    fn append(&mut self, child: &ElementRef) {
        child.element().__parent = Some(self.element());
        self.element()
            .child_nodes
            .push(HtmlNode::Element(child.clone()));
    }

    fn prepend(&mut self, child: &ElementRef) {
        child.element().__parent = Some(self.element());
        self.element()
            .child_nodes
            .insert(0, HtmlNode::Element(child.clone()));
    }

    fn has_children(&self) -> bool {
        self.element().child_nodes.len() > 0
    }

    fn remove(&mut self, node: &ElementRef) {
        self.element()
            .child_nodes
            .retain(|n| match &n.as_element_ref() {
                Some(r) => r != node,
                None => true,
            });
        node.element().__parent = None;
    }

    fn after(&mut self, node: &ElementRef) {
        if let Some(value) = &self.parent() {
            let index = value.get_index_of(self).unwrap();
            value
                .element()
                .child_nodes
                .insert(index + 1, HtmlNode::Element(node.clone()));
        }
    }

    fn children(&self) -> impl Iterator<Item = &ElementRef> {
        self.element()
            .child_nodes
            .iter()
            .filter(|node| node.is_element())
            .map(|node| match node {
                HtmlNode::Element(e) => e,
                _ => todo!(),
            })
    }
}

impl Query<'_> for ElementRef {}

impl PartialEq for ElementRef {
    fn eq(&self, other: &Self) -> bool {
        self.__element == other.__element
    }
}

impl Drop for ElementRef {
    fn drop(&mut self) {
        self.element().refs -= 1;
        unsafe {
            if self.element().refs == 0 {
                std::mem::drop(Box::from_raw(self.__element));
            }
        }
    }
}

impl Clone for ElementRef {
    fn clone(&self) -> Self {
        self.element().refs += 1;
        Self {
            __element: self.__element,
        }
    }
}
