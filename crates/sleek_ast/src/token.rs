use crate::Span;

use super::HtmlTag;

#[derive(Debug, PartialEq)]
pub enum HtmlToken {
    DocType {
        name: String,
        force_quirks: bool,
    },
    OpeningTag {
        name: HtmlTag,
        attributes: Vec<HtmlAttribute>,
        location: Span,
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
        location: Span,
    },
    EOF {
        location: [usize; 2],
    },
}

#[derive(Debug, PartialEq)]
pub struct HtmlAttribute {
    pub key: String,
    pub value: Option<String>,
    pub quoted: AttributeQuoteType,
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
