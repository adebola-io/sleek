#![allow(unused)]

use std::fmt::Debug;

use sleek_utils::MutableCountRef;

use super::ElementRef;

#[derive(Clone, PartialEq)]
pub struct Span {
    pub start: [usize; 2],
    pub end: [usize; 2],
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{:?}, {:?}]", self.start, self.end))
    }
}

impl Span {
    /// Create a span with the given range.
    pub fn over(start: [usize; 2], end: [usize; 2]) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementSpan {
    pub open_tag: Span,
    pub close_tag: Option<Span>,
}

impl ElementSpan {
    pub fn empty() -> Self {
        ElementSpan {
            open_tag: Span {
                start: [0, 0],
                end: [0, 0],
            },
            close_tag: None,
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
    pub span: Span,
}

#[derive(Debug)]
pub struct DocRef {
    doctype: MutableCountRef<HtmlDocType>,
}

#[derive(Debug)]
pub struct HtmlComment {
    pub content: String,
    pub span: Span,
}

pub enum HtmlNode {
    DocType(DocRef),
    Text(HtmlTextNode),
    Element(ElementRef),
    Comment(HtmlComment),
}

impl Debug for HtmlNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DocType(arg0) => arg0.fmt(f),
            Self::Text(arg0) => arg0.fmt(f),
            Self::Element(arg0) => arg0.fmt(f),
            Self::Comment(arg0) => arg0.fmt(f),
        }
    }
}

impl HtmlNode {
    pub fn as_element_ref(&self) -> Option<ElementRef> {
        match self {
            HtmlNode::Element(el) => Some(el.clone()),
            _ => None,
        }
    }

    /// Returns `true` if the html node is [`Element`].
    ///
    /// [`Element`]: HtmlNode::Element
    #[must_use]
    pub fn is_element(&self) -> bool {
        matches!(self, Self::Element(..))
    }
}
