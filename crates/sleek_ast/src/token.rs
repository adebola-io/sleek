use crate::Span;

use super::HtmlTag;

#[derive(Debug, PartialEq)]
pub enum HtmlToken {
    DocType {
        root: String,
        identifier: Option<DocTypeIdentifier>,
    },
    OpeningTag {
        name: HtmlTag,
        attributes: Vec<HtmlAttribute>,
        span: Span,
        self_closing: bool,
    },
    ClosingTag {
        name: HtmlTag,
        span: Span,
    },
    Text {
        content: String,
        span: Span,
    },
    CharacterRef {
        r#type: CharacterRef,
        location: Span,
    },
    Comment {
        content: String,
        span: Span,
    },
    EOF {
        location: [usize; 2],
    },
}

#[derive(Debug, PartialEq)]
pub struct HtmlAttribute {
    pub key: String,
    pub value: Option<String>,
    pub quote_type: AttributeQuoteType,
}

#[derive(Debug, PartialEq)]
pub enum CharacterRef {
    Amp,
}

#[derive(Debug, PartialEq)]
pub enum AttributeQuoteType {
    Single,
    Double,
    None,
}

#[derive(Debug, PartialEq)]
pub enum DocTypeIdentifier {
    System,
    Public,
}
