use std::str::Chars;

use sleek_utils::QueueMatrix;

use crate::{
    html::{
        tokenizer::{tokenize, TokenStore},
        HtmlParseErrorType,
    },
    HtmlParseResult,
};

use sleek_ast::{ElementRef, HtmlAttribute, HtmlDocument, HtmlTag, HtmlToken, Span};

use crate::HtmlParseError;

pub struct SpeculativeHtmlParser;

impl SpeculativeHtmlParser {
    pub fn parse(
        mut token_store: TokenStore,
        mut iterator: QueueMatrix<Chars<'_>>,
    ) -> HtmlParseResult {
        let parser = Box::into_raw(Box::new(Parser::new()));
        token_store.on_token_input(Box::new(move |token| unsafe { (*parser).receive(token) }));
        tokenize(&mut token_store, &mut iterator);
        unsafe {
            let result = (*parser).finish(token_store.errors);
            std::mem::drop(Box::from_raw(parser));
            result
        }
    }
}
#[allow(dead_code)]
pub enum ParserResponse {
    SwitchToStyleSheet,
    SwitchToScript,
    Continue,
}

/// A parser that constructs the document tree bit by bit from a stream of tokens.
struct Parser {
    tree: HtmlDocument,
    current_element: Option<ElementRef>,
    store: Vec<HtmlToken>,
    errors: Vec<HtmlParseError>,
}

impl Parser {
    fn new() -> Self {
        Self {
            tree: HtmlDocument { nodes: vec![] },
            current_element: None,
            store: vec![],
            errors: vec![],
        }
    }
    fn receive(&mut self, token: HtmlToken) -> ParserResponse {
        match token {
            HtmlToken::OpeningTag {
                name,
                attributes,
                span,
                self_closing,
            } => {
                if name == HtmlTag::Script {
                    ParserResponse::SwitchToScript
                } else if name == HtmlTag::Style {
                    ParserResponse::SwitchToStyleSheet
                } else {
                    self.parse_opening_tag(name, attributes, span, self_closing);
                    ParserResponse::Continue
                }
            }
            _ => todo!(),
        }
    }
    fn parse_opening_tag(
        &mut self,
        name: HtmlTag,
        attributes: Vec<HtmlAttribute>,
        span: Span,
        self_closing: bool,
    ) {
        let is_void = name.is_void();

        let mut element = ElementRef::init(name, attributes, span);

        // Elements that are not void cannot be self closing. Not a fatal error.
        if self_closing && !is_void {
            self.errors.push(HtmlParseError {
                error_type: HtmlParseErrorType::SelfClosingNonVoidTag,
                location: element.get_end(),
            });
        }

        // element
    }
    fn finish(&mut self, mut tokenizer_errors: Vec<HtmlParseError>) -> HtmlParseResult {
        tokenizer_errors.append(&mut self.errors);
        HtmlParseResult {
            tree: std::mem::take(&mut self.tree),
            errors: tokenizer_errors,
        }
    }
}
