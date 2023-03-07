use sleek_utils::Node;

use crate::{ElementRef, HtmlNode, Query};

#[derive(Debug)]
pub struct HtmlDocument {
    pub nodes: Vec<HtmlNode>,
}

impl HtmlDocument {
    pub fn new() -> Self {
        HtmlDocument { nodes: vec![] }
    }
}

impl<'a> Node<'a, ElementRef> for HtmlDocument {
    fn parent(&self) -> Option<ElementRef> {
        None
    }

    fn children(&'a self) -> impl Iterator<Item = &ElementRef> {
        self.nodes
            .iter()
            .filter(|node| node.is_element())
            .map(|node| match node {
                HtmlNode::Element(e) => e,
                _ => unreachable!(),
            })
    }

    fn append(&mut self, child: &ElementRef) {
        self.nodes.push(HtmlNode::Element(child.clone()));
    }

    fn prepend(&mut self, child: &ElementRef) {
        self.nodes.insert(0, HtmlNode::Element(child.clone()));
    }

    fn has_children(&self) -> bool {
        self.nodes.len() > 0
    }

    fn remove(&mut self, node: &ElementRef) {
        self.nodes.retain(|n| match &n.as_element_ref() {
            Some(r) => r != node,
            None => true,
        });
    }

    fn after(&mut self, _: &ElementRef) {
        panic!("HTMLTreeException: Only one node is allowed at the root")
    }
}

impl Query<'_> for HtmlDocument {}
