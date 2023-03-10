#![allow(unused)]

use std::{cell::RefCell, mem::take, rc::Rc, str::Chars};

use sleek_utils::{Node, QueueMatrix};

use crate::{
    html::{
        tokenizer::{tokenize, TokenStore},
        HtmlParseErrorType,
    },
    HtmlParseResult,
};

use sleek_ast::{
    ElementRef, HtmlAttribute, HtmlComment, HtmlDocType, HtmlDocument, HtmlNode, HtmlTag,
    HtmlTextNode, HtmlToken, Span,
};

use crate::HtmlParseError;

pub struct SpeculativeHtmlParser;

impl SpeculativeHtmlParser {
    pub fn parse(
        mut token_store: TokenStore,
        mut iterator: QueueMatrix<Chars<'_>>,
    ) -> HtmlParseResult {
        unsafe {
            let parser = Box::into_raw(Box::new(Parser::new()));
            token_store.on_token_input(Box::new(move |token| (*parser).receive(token)));
            tokenize(&mut token_store, &mut iterator);
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
    open_tags: usize,
    errors: Vec<HtmlParseError>,
}

impl Parser {
    fn new() -> Self {
        Self {
            tree: HtmlDocument { nodes: vec![] },
            current_element: None,
            store: vec![],
            open_tags: 0,
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
            HtmlToken::ClosingTag { name, span } => {
                self.parse_closing_tag(name, span);
                ParserResponse::Continue
            }
            HtmlToken::Text { content, span } => {
                self.parse_text(content, span);
                ParserResponse::Continue
            }
            HtmlToken::Comment { content, span } => {
                self.parse_comment(content, span);
                ParserResponse::Continue
            }
            HtmlToken::DocType {
                name,
                r#type,
                force_quirks,
            } => {
                self.tree.nodes.push(HtmlNode::DocType(HtmlDocType {
                    name,
                    r#type,
                    force_quirks,
                }));
                ParserResponse::Continue
            }
            _ => unreachable!(),
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

        let mut new_element = ElementRef::init(name, attributes, span);

        // Elements that are not void cannot be self closing. Not a fatal error.
        if self_closing && !is_void {
            self.errors.push(HtmlParseError {
                error_type: HtmlParseErrorType::SelfClosingNonVoidTag,
                location: new_element.get_end(),
            });
        }

        match &mut self.current_element {
            Some(element) => {
                element.append(&new_element);
            }
            // If there is no parent, treat as root element.
            None => {
                self.tree.append(&new_element);
            }
        }

        if !self_closing && !is_void {
            // Expect element's children or closing tag.
            self.current_element = Some(new_element);
            self.open_tags += 1;
        }

        // element
    }
    fn parse_closing_tag(&mut self, name: HtmlTag, span: Span) {
        match &mut self.current_element {
            Some(current_element) => {
                if &name == current_element.tag_name() {
                    current_element.element().location.close_tag = Some(span);
                    // Go back up one level.
                    self.current_element = current_element.parent();
                    self.open_tags -= 1;
                } else {
                    self.errors.push(HtmlParseError {
                        error_type: HtmlParseErrorType::UnexpectedCloseTag(name),
                        location: span.start,
                    });
                }
            }
            None => self.errors.push(HtmlParseError {
                error_type: HtmlParseErrorType::UnexpectedCloseTag(name),
                location: span.start,
            }),
        }
    }

    /// Add a text node to the tree.
    fn parse_text(&mut self, content: String, span: Span) {
        let text_node = HtmlTextNode { content, span };
        match &mut self.current_element {
            Some(current) => current.append_text(text_node),
            None => self.tree.nodes.push(HtmlNode::Text(text_node)),
        }
    }

    fn parse_comment(&mut self, content: String, span: Span) {
        let node = HtmlNode::Comment(HtmlComment { content, span });
        match &mut self.current_element {
            Some(current) => current.element().child_nodes.push(node),
            None => self.tree.nodes.push(node),
        }
    }

    fn finish(&mut self, mut tokenizer_errors: Vec<HtmlParseError>) -> HtmlParseResult {
        // check for unclosed tags.
        if self.open_tags != 0 {
            let current_open_subtree = self.current_element.as_ref().unwrap();
            self.errors.push(HtmlParseError {
                error_type: HtmlParseErrorType::UnclosedTag(
                    current_open_subtree.tag_name().clone(),
                ),
                location: current_open_subtree.get_end(),
            });
        }
        tokenizer_errors.append(&mut self.errors);

        HtmlParseResult {
            tree: std::mem::take(&mut self.tree),
            errors: tokenizer_errors,
        }
    }
}
