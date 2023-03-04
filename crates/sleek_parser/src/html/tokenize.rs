use std::{mem::take, str::Chars};

use sleek_ast::{HtmlTag, HtmlToken, Span};
use sleek_utils::{HigherOrderIterator, MatrixIterator, StackIterator};

use crate::HtmlParseError;

use super::error::HtmlParseErrorType as ErrorType;

enum State {
    Data,
    TagStart,
    ClosingTagStart,
    ClosingTagEnd,
}
enum Event {
    Text,
    Close,
}

pub struct TokenizerResult {
    pub tokens: Vec<HtmlToken>,
    pub errors: Vec<HtmlParseError>,
    has_data: bool,
    cache: String,
    loc: [usize; 2],
}

impl TokenizerResult {
    /// Store a character in the cache.
    fn push(&mut self, ch: char) {
        if !self.has_data {
            self.has_data = true
        };
        self.cache.push(ch);
    }
    //  fn emit_or_error(&mut self, event: Event, end: [usize; 2]) {

    //      //
    //  }
    /// Push a token to the token list.
    fn emit(&mut self, event: Event, iterator: &StackIterator<MatrixIterator<Chars<'_>>>) {
        if self.has_data {
            let content = take(&mut self.cache);
            let span = Span::over(self.loc, iterator.inner().get_position());

            let token = match event {
                Event::Text => HtmlToken::Text { content, span },
                Event::Close => HtmlToken::ClosingTag {
                    name: HtmlTag::new(content),
                    span,
                },
            };

            self.tokens.push(token);
            self.has_data = false;
        }
    }
    fn error(
        &mut self,
        error_type: ErrorType,
        iterator: &StackIterator<MatrixIterator<Chars<'_>>>,
    ) {
        self.errors.push(HtmlParseError {
            error_type,
            location: iterator.inner().get_position(),
        });
    }
    fn set_start(&mut self, iterator: &StackIterator<MatrixIterator<Chars<'_>>>) {
        self.loc = iterator.inner().get_position();
        self.loc[1] -= 1;
    }
    fn empty(&self) -> bool {
        !self.has_data
    }
}

pub fn tokenize_html(input: &str) -> TokenizerResult {
    let mut iterator = StackIterator::new(MatrixIterator::new(input.chars(), '\n'));
    let mut state = State::Data;
    let mut store = TokenizerResult {
        tokens: vec![],
        errors: vec![],
        has_data: false,
        loc: [0, 0],
        cache: String::new(),
    };

    loop {
        match state {
            // Parse regular html text, without any formatting.
            State::Data => match iterator.next() {
                Some('<') => {
                    store.emit(Event::Text, &iterator);
                    store.set_start(&iterator);
                    state = State::TagStart
                }
                Some(ch) => {
                    // Collect the starting point of the text node.
                    if store.empty() {
                        store.set_start(&iterator)
                    }
                    store.push(ch)
                }
                None => {
                    store.emit(Event::Text, &iterator);
                    break;
                }
            },
            // A tag has been opened.
            State::TagStart => match iterator.next() {
                Some('/') => state = State::ClosingTagStart,
                Some(_) => todo!(),
                None => {
                    store.error(ErrorType::UnexpextedEndOfFile, &iterator);
                    // Emit as text.
                    store.push('<');
                    store.emit(Event::Text, &iterator);
                    break;
                }
            },
            // A closing tag has been opened.
            State::ClosingTagStart => match iterator.next() {
                Some(ch) if ch.is_whitespace() => {
                    // tagnames must directly follow the </
                    if store.empty() {
                        store.error(ErrorType::ExpectedTagName, &iterator);
                        state = State::Data;
                    }
                }
                Some(ch @ ('A'..='Z' | 'a'..='z' | '0'..='9')) => {
                    // closing tags cannot start with numbers.
                    if store.empty() && ch.is_numeric() {
                        store.error(ErrorType::UnexpectedCharacter, &iterator);
                        iterator.push(ch);
                        state = State::Data;
                    } else {
                        // Collect character for html tagname.
                        store.push(ch.to_ascii_lowercase());
                    }
                }
                Some('>') => {
                    // Tag was closed without any name.
                    if store.empty() {
                        store.error(ErrorType::ExpectedTagName, &iterator);
                    } else {
                        store.emit(Event::Close, &iterator);
                        state = State::Data
                    }
                }
                Some(ch) => {
                    store.error(ErrorType::UnexpectedCharacter, &iterator);
                    iterator.push(ch);
                    state = State::Data;
                }
                None => {
                    store.error(ErrorType::UnexpextedEndOfFile, &iterator);
                }
            },
            _ => todo!(),
        }
    }

    store
}
