#![allow(unused)]

use sleek_utils::MutableCountRef;

use super::ElementRef;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: [usize; 2],
    pub end: [usize; 2],
}

impl Span {
    /// Create a span with the given range.
    pub fn over(start: [usize; 2], end: [usize; 2]) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementSpan {
    open_tag: Span,
    close_tag: Span,
}

impl ElementSpan {
    pub fn empty() -> Self {
        ElementSpan {
            open_tag: Span {
                start: [0, 0],
                end: [0, 0],
            },
            close_tag: Span {
                start: [0, 0],
                end: [0, 0],
            },
        }
    }
}

#[derive(Debug)]
pub struct HtmlDocType {
    name: String,
    force_quirks: bool,
}

#[derive(Debug)]
pub struct HtmlTextNode {
    pub content: String,
    _parent: Option<usize>,
    pub location: ElementSpan,
}

#[derive(Debug)]
pub struct TextRef {
    pub text: MutableCountRef<HtmlTextNode>,
}

#[derive(Debug)]
pub struct CommentRef {
    comment: MutableCountRef<HtmlComment>,
}

#[derive(Debug)]
pub struct DocRef {
    doctype: MutableCountRef<HtmlDocType>,
}

#[derive(Debug)]
pub struct HtmlComment {
    pub content: String,
    _parent: Option<usize>,
    pub location: ElementSpan,
}

#[derive(Debug)]
pub enum HtmlNode {
    DocType(DocRef),
    Text(TextRef),
    Element(ElementRef),
    Comment(CommentRef),
}

impl HtmlNode {
    pub fn as_element_ref(&self) -> Option<ElementRef> {
        match self {
            HtmlNode::Element(el) => Some(el.clone()),
            _ => None,
        }
    }
}
