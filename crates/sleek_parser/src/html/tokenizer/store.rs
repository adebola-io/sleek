use std::{mem::take, str::Chars};

use sleek_ast::{
    AttributeQuoteType as QuoteType, DocTypeIdentifier, HtmlAttribute, HtmlTag, HtmlToken, Span,
};
use sleek_utils::{HigherOrderIterator, MatrixIterator, QueueIterator, QueueMatrix};

use crate::{
    html::{error::HtmlParseErrorType as ErrorType, parser::ParserResponse},
    HtmlParseError,
};

pub enum Event {
    Text,
    Close,
    Comment,
    OpenerTag(bool),
    DocType(String, Option<DocTypeIdentifier>),
}

pub struct TokenStore {
    pub tokens: Vec<HtmlToken>,
    pub errors: Vec<HtmlParseError>,
    has_data: bool,
    attrib_store: Vec<HtmlAttribute>,
    pub cache: (String, String, Option<String>),
    loc: [usize; 2],
    listener: Option<Box<dyn Fn(HtmlToken) -> ParserResponse>>,
}

impl TokenStore {
    /// Store a character in the cache.
    pub fn push(&mut self, ch: char) {
        if !self.has_data {
            self.has_data = true
        };
        self.cache.0.push(ch);
    }
    pub fn push_str(&mut self, st: &str) {
        self.cache.0.push_str(st);
    }
    /// Push a character into an attribute name.
    pub fn push_attr_name(&mut self, ch: char) {
        self.cache.1.push(ch);
    }
    /// Push a character into the attribute value, if it exists, or create a new attribute value to push into if it doesn't
    pub fn push_attr_value(&mut self, ch: char) {
        match &mut self.cache.2 {
            Some(string) => string.push(ch),
            None => {
                let mut string = String::new();
                string.push(ch);
                self.cache.2 = Some(string);
            }
        }
    }
    pub fn collect_attribute(&mut self, quote_type: QuoteType) {
        self.attrib_store.push(HtmlAttribute {
            key: take(&mut self.cache.1),
            value: self.cache.2.take(),
            quote_type,
        })
    }
    /// Set a listener for emission events.
    pub fn on_token_input(&mut self, f: Box<dyn Fn(HtmlToken) -> ParserResponse>) {
        self.listener = Some(f);
    }
    /// Push a token to the token list.
    pub fn emit(&mut self, event: Event, iterator: &mut QueueMatrix<Chars<'_>>) {
        let content = take(&mut self.cache.0);
        self.has_data = false;
        let mut span = Span::over(self.loc, iterator.inner().locus());

        let token = match event {
            Event::Text => {
                // Ignore empty text nodes.
                if content.find(|ch: char| !ch.is_whitespace()).is_none() {
                    return;
                }
                span.end[1] -= 1;
                HtmlToken::Text { content, span }
            }
            Event::OpenerTag(self_closing) => {
                let attributes = take(&mut self.attrib_store);
                HtmlToken::OpeningTag {
                    name: HtmlTag::new(content),
                    attributes,
                    span,
                    self_closing,
                }
            }
            Event::Close => HtmlToken::ClosingTag {
                name: HtmlTag::new(content),
                span,
            },
            Event::Comment => HtmlToken::Comment { content, span },
            Event::DocType(root, identifier) => HtmlToken::DocType { root, identifier },
        };

        match &self.listener {
            Some(listener) => match listener(token) {
                ParserResponse::SwitchToStyleSheet => todo!(),
                ParserResponse::SwitchToScript => todo!(),
                ParserResponse::Continue => {}
            },
            None => self.tokens.push(token),
        }
    }
    /// Adds an error.
    pub fn error(
        &mut self,
        error_type: ErrorType,
        iterator: &QueueIterator<MatrixIterator<Chars<'_>>>,
    ) {
        let location = iterator.inner().locus();
        self.errors.push(HtmlParseError {
            error_type,
            location,
        });
    }
    /// Sets the position of the iterator to the start of something.
    pub fn set_start(&mut self, iterator: &QueueIterator<MatrixIterator<Chars<'_>>>) {
        self.loc = iterator.inner().locus();
        self.loc[1] -= 1;
    }
    /// Checks if the store contains data in its cache.
    pub fn empty(&self) -> bool {
        !self.has_data
    }
    /// Removes data from the store cache.
    pub fn clear(&mut self) {
        self.cache.0.clear();
        self.cache.1.clear();
        self.cache.2 = None;
    }
}

impl TokenStore {
    /// Create a new tokenizer.
    pub fn new() -> Self {
        TokenStore {
            tokens: vec![],
            errors: vec![],
            attrib_store: vec![],
            has_data: false,
            loc: [0, 0],
            cache: (String::new(), String::new(), None),
            listener: None,
        }
    }
}
