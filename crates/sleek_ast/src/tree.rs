use sleek_utils::{MutableCountRef, Node};

use crate::{ElementRef, HtmlNode, Query};

pub struct HtmlTree {
    nodes: MutableCountRef<Vec<HtmlNode>>,
}

impl Node<ElementRef> for HtmlTree {
    fn parent(&self) -> Option<ElementRef> {
        None
    }

    fn children(&self) -> Vec<ElementRef> {
        let mut children = vec![];
        for node in self.nodes.borrow().iter() {
            if let Some(element_ref) = node.as_element_ref() {
                children.push(element_ref);
            }
        }
        children
    }

    fn append(&mut self, child: &ElementRef) {
        self.nodes
            .borrow_mut()
            .push(HtmlNode::Element(child.clone()));
    }

    fn prepend(&mut self, child: &ElementRef) {
        self.nodes
            .borrow_mut()
            .insert(0, HtmlNode::Element(child.clone()));
    }

    fn remove(&mut self, node: &ElementRef) {
        self.nodes
            .borrow_mut()
            .retain(|n| match &n.as_element_ref() {
                Some(r) => r != node,
                None => true,
            });
    }

    fn after(&mut self, _: &ElementRef) {
        panic!("HTMLTreeException: Only one node is allowed at the root")
    }
}

impl Query for HtmlTree {}
