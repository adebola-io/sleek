use std::{cell::RefCell, mem::take, rc::Rc};

use sleek_ast::{
    ElementRef, HtmlAttribute, HtmlComment, HtmlDocument, HtmlNode, HtmlTag, HtmlTextNode,
    HtmlToken, Span, TextRef,
};

use crate::HtmlParseError;

use super::{error::HtmlParseErrorType, tokenize::HtmlTokenizer};

type FallibleStep<T> = Result<T, HtmlParseError>;

pub struct HtmlParser {
    tokens: Vec<HtmlToken>,
    index: usize,
    /// The parser removes tokens from the beginning of the token array when creating a tree.
    /// Since it uses swap_remove, the rev_separator indicates where the accessor should stop and start moving backwards to collect swapped values.
    rev_separator: usize,
    errors: Vec<HtmlParseError>,
}

/// The result of the Html parsing process.
/// The parser tries to produce a valid HTML DOM Tree regardless of how wrangled or broken the input string is.
/// The resulting tree, as well as errors encountered, are stored on this struct.
#[derive(Debug)]
pub struct HtmlParseResult {
    pub tree: HtmlDocument,
    pub errors: Vec<HtmlParseError>,
}

impl HtmlParser {
    /// Analyse an array of tokens into a document tree.
    pub fn parse(mut tokenizer: HtmlTokenizer) -> HtmlParseResult {
        let rev_separator = tokenizer.tokens.len() >> 1;
        let mut parser = Self {
            tokens: take(&mut tokenizer.tokens),
            index: 0,
            rev_separator,
            errors: take(&mut tokenizer.errors),
        };

        let mut nodes = vec![];

        while let Some(token) = parser.next() {
            if !token.is_eof() {
                match parser.parse_node(token) {
                    Ok(node) => nodes.push(node),
                    Err(err) => parser.errors.push(err),
                }
            }
        }

        HtmlParseResult {
            tree: HtmlDocument { nodes },
            errors: take(&mut parser.errors),
        }
    }

    /// Parse the next token into a node.
    fn parse_node(&mut self, token: HtmlToken) -> FallibleStep<HtmlNode> {
        match token {
            HtmlToken::OpeningTag {
                name,
                attributes,
                span,
                self_closing,
            } => Ok(HtmlNode::Element(self.create_element(
                name,
                attributes,
                span,
                self_closing,
            ))),
            HtmlToken::Text { content, span } => Ok(self.create_text_node(content, span)),
            // Stray closing tag.
            HtmlToken::ClosingTag { name, span } => Err(HtmlParseError {
                error_type: HtmlParseErrorType::UnexpectedCloseTag(name),
                location: span.start,
            }),
            HtmlToken::Comment { content, span } => Ok(self.create_comment_node(content, span)),
            _ => {
                println!("{:?}", token);
                todo!()
            }
        }
    }

    /// Start parsing a new element.
    fn create_element(
        &mut self,
        name: HtmlTag,
        attributes: Vec<HtmlAttribute>,
        span: Span,
        self_closing: bool,
    ) -> ElementRef {
        let is_void = name.is_void();

        let mut element = ElementRef::init(name, attributes, span);

        // Elements that are not void cannot be self closing. Not a fatal error.
        if self_closing && !is_void {
            self.errors.push(HtmlParseError {
                error_type: HtmlParseErrorType::SelfClosingNonVoidTag,
                location: element.get_end(),
            });
        }
        if !(self_closing || is_void) {
            // Parse element's children if it is valid.
            self.parse_children(&mut element);
        };

        element
    }

    /// Attempt to parse a node's children.
    fn parse_children(&mut self, parent_element: &mut ElementRef) {
        loop {
            match self.next() {
                Some(token) => match token {
                    // Tag was unclosed.
                    HtmlToken::EOF { location } => {
                        self.errors.push(HtmlParseError {
                            error_type: HtmlParseErrorType::UnclosedTag(
                                parent_element.tag_name().clone(),
                            ),
                            location,
                        });
                        break;
                    }
                    // Closing tag for parent encountered.
                    HtmlToken::ClosingTag { name, span } if &name == parent_element.tag_name() => {
                        parent_element.element().location.close_tag = Some(span);
                        break;
                    }
                    _ => match self.parse_node(token) {
                        Ok(node) => parent_element.element().child_nodes.push(node),
                        Err(err) => self.errors.push(err),
                    },
                },
                None => unreachable!(),
            }
        }
    }

    fn create_text_node(&self, content: String, span: Span) -> HtmlNode {
        let node = HtmlTextNode { content, span };
        HtmlNode::Text(TextRef {
            text: Rc::new(RefCell::new(node)),
        })
    }

    fn create_comment_node(&self, content: String, span: Span) -> HtmlNode {
        let comment = HtmlComment { content, span };
        HtmlNode::Comment(comment)
    }
}

impl Iterator for HtmlParser {
    type Item = HtmlToken;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.tokens.len();
        // Front removal magic.
        if len == 0 {
            None
        } else {
            Some(self.tokens.swap_remove(if len > self.rev_separator {
                self.index += 1;
                self.index - 1
            } else {
                self.rev_separator -= 1;
                self.rev_separator
            }))
        }
    }
}
