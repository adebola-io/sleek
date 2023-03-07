use std::{mem::take, str::Chars};

use sleek_ast::{
    AttributeQuoteType as QuoteType, DocTypeIdentifier, HtmlAttribute, HtmlTag, HtmlToken, Span,
};
use sleek_utils::{HigherOrderIterator, MatrixIterator, QueueIterator};

use crate::HtmlParseError;

use super::error::HtmlParseErrorType as ErrorType;

#[derive(Debug)]
enum State {
    Data,
    OpeningTag,
    ClosingTag,
    BogusComment,
    AttributeName,
    Comment,
    AttributeValue,
    Doctype,
}
enum Event {
    Text,
    Close,
    Comment,
    OpenerTag(bool),
    DocType(String, Option<DocTypeIdentifier>),
}

pub struct HtmlTokenizer {
    pub tokens: Vec<HtmlToken>,
    pub errors: Vec<HtmlParseError>,
    has_data: bool,
    attrib_store: Vec<HtmlAttribute>,
    cache: (String, String, Option<String>),
    loc: [usize; 2],
}

impl HtmlTokenizer {
    /// Store a character in the cache.
    fn push(&mut self, ch: char) {
        if !self.has_data {
            self.has_data = true
        };
        self.cache.0.push(ch);
    }
    fn push_str(&mut self, st: &str) {
        self.cache.0.push_str(st);
    }
    /// Push a character into an attribute name.
    fn push_attr_name(&mut self, ch: char) {
        self.cache.1.push(ch);
    }
    /// Push a character into the attribute value, if it exists, or create a new attribute value to push into if it doesn't
    fn push_attr_value(&mut self, ch: char) {
        match &mut self.cache.2 {
            Some(string) => string.push(ch),
            None => {
                let mut string = String::new();
                string.push(ch);
                self.cache.2 = Some(string);
            }
        }
    }
    fn collect_attribute(&mut self, quote_type: QuoteType) {
        self.attrib_store.push(HtmlAttribute {
            key: take(&mut self.cache.1),
            value: self.cache.2.take(),
            quote_type,
        })
    }
    /// Push a token to the token list.
    fn emit(&mut self, event: Event, iterator: &QueueIterator<MatrixIterator<Chars<'_>>>) {
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

        self.tokens.push(token);
    }
    /// Adds an error.
    fn error(
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
    fn set_start(&mut self, iterator: &QueueIterator<MatrixIterator<Chars<'_>>>) {
        self.loc = iterator.inner().locus();
        self.loc[1] -= 1;
    }
    /// Checks if the store contains data in its cache.
    fn empty(&self) -> bool {
        !self.has_data
    }
    /// Removes data from the store cache.
    fn clear(&mut self) {
        self.cache.0.clear();
        self.cache.1.clear();
        self.cache.2 = None;
    }
}

pub fn tokenize_html(input: &str) -> HtmlTokenizer {
    let mut iterator = QueueIterator::new(MatrixIterator::new(input.chars(), '\n'));
    let mut state = State::Data;
    let mut store = HtmlTokenizer {
        tokens: vec![],
        errors: vec![],
        attrib_store: vec![],
        has_data: false,
        loc: [0, 0],
        cache: (String::new(), String::new(), None),
    };

    // iterator.on_push(Rc::new(|inner| {
    //     println!("Shifting from {:?}", inner.locus());
    //     inner.left();
    //     println!("Shifted to {:?}", inner.locus());
    // }));

    loop {
        match state {
            // Parse regular html text, without any formatting.
            State::Data => match iterator.next() {
                Some('<') => {
                    if !store.empty() {
                        store.emit(Event::Text, &iterator);
                    }
                    store.set_start(&iterator);
                    state = State::OpeningTag
                }
                Some(ch) => {
                    // Collect the starting point of the text node.
                    if store.empty() {
                        store.set_start(&iterator)
                    }
                    store.push(ch)
                }
                None => {
                    if !store.empty() {
                        store.emit(Event::Text, &iterator);
                    }
                    break;
                }
            },
            // A tag has been opened.
            State::OpeningTag => match iterator.next() {
                Some('/') => {
                    // There is nothing between < and /. Parse as closing tag.
                    if store.empty() {
                        state = State::ClosingTag
                    } else {
                        // Open tag is possibly self-closing.
                        loop {
                            match iterator.next() {
                                // Skip whitespace.
                                Some(ch) if ch.is_whitespace() => {}
                                // tag is self-closing.
                                Some('>') => {
                                    store.emit(Event::OpenerTag(true), &iterator);
                                    state = State::Data;
                                    break;
                                }
                                // Parse error. Scan character again as attribute.
                                Some(ch) => {
                                    store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                                    iterator.push(ch);
                                    state = State::AttributeName;
                                    break;
                                }
                                // Tag was unclosed.
                                None => {
                                    store.clear();
                                    store.error(ErrorType::UnexpectedEndOfInput, &iterator);
                                    state = State::Data;
                                    break;
                                }
                            }
                        }
                    }
                }
                Some('!') => {
                    if store.empty() {
                        // Comment or Doctype tag.
                        // Collect next two characters and check if they match commment start (--).
                        match iterator.next() {
                            // First -
                            Some('-') => match iterator.next() {
                                // Second -
                                Some('-') => state = State::Comment,
                                // Any other character. Take first - as part of the comment.
                                Some(ch) => {
                                    store.push('-');
                                    store.push(ch);
                                    store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                                    state = State::Comment
                                }
                                None => {
                                    store.push('-');
                                    store.emit(Event::Comment, &iterator);
                                    store.error(ErrorType::UnexpectedEndOfInput, &iterator);
                                    break;
                                }
                            },
                            // Default to comment.
                            None => {
                                store.emit(Event::Comment, &iterator);
                                store.error(ErrorType::UnexpectedEndOfInput, &iterator);
                                break;
                            }
                            // Check for !doctype
                            Some(ch @ ('d' | 'D')) => {
                                let value: String = iterator.collect_next(6);
                                if value.to_ascii_lowercase() == "octype" {
                                    state = State::Doctype;
                                } else {
                                    println!("{value}");
                                    store.push(ch);
                                    store.push_str(value.as_str());
                                    store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                                    state = State::Comment;
                                }
                            }
                            Some(ch) => {
                                store.push(ch);
                                store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                                state = State::BogusComment;
                            }
                        }
                    } else {
                        // Tagnames has already started parsing.
                        store.push('!');
                    }
                }
                Some('>') => {
                    // Parse <> as text.
                    if store.empty() {
                        store.push('<');
                        iterator.push('>');
                        store.error(ErrorType::UnexpectedCharacter('>'), &iterator);
                        state = State::Data;
                    } else {
                        // Push an opening tag with no attributes.
                        store.emit(Event::OpenerTag(false), &iterator);
                        state = State::Data;
                    }
                }
                Some(ch) if ch.is_ascii_alphanumeric() || ch == '-' => {
                    // Tags cannot start with numeric values. Reparse the tag as plain text.
                    if store.empty() && ch.is_numeric() {
                        store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                        store.push('<');
                        store.push(ch);
                        state = State::Data;
                    } else {
                        store.push(ch.to_ascii_lowercase());
                    }
                }
                Some(ch) if ch.is_whitespace() => {
                    // Parse error. Expected a tagname to be present. Reparse tag as text.
                    if store.empty() {
                        store.push('<');
                        iterator.push(ch);
                        store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                        state = State::Data;
                    } else {
                        // Parsing an attribute. Revisit the current character as an attribute name.
                        iterator.push(ch);
                        state = State::AttributeName;
                    }
                }
                // Invalid character.
                Some(_) => {}
                None => {
                    store.error(ErrorType::UnexpectedEndOfInput, &iterator);
                    // Emit as text.
                    store.push('<');
                    store.emit(Event::Text, &iterator);
                    break;
                }
            },
            // Parsing an attribute name.
            State::AttributeName => {
                let mut ended = false;
                let mut has_value = false;
                // Collect characters until a = is encountered or the input ends.
                while !(ended || has_value) {
                    match iterator.next() {
                        Some(ch) if ch.is_whitespace() => {
                            // Skip over succeeding whitespaces.
                            iterator.next_until(|ch| !ch.is_whitespace());
                            while !ended {
                                match iterator.next() {
                                    Some(ch) => {
                                        // Another attribute encountered. Reparse as attribute name.
                                        iterator.push(ch);
                                        break;
                                    }
                                    // Input ended without tag close.
                                    None => ended = true,
                                }
                            }
                            break;
                        }
                        // Expect attribute value.
                        Some('=') => has_value = true,
                        // End of tag. Parse end in tag state.
                        Some(ch @ ('>' | '/')) => {
                            iterator.push(ch);
                            state = State::OpeningTag;
                            break;
                        }
                        // Push character into name.
                        Some(ch) => store.push_attr_name(ch),
                        // Input ends abruptly.
                        None => ended = true,
                    }
                }
                // = encountered. attribute equals something.
                if has_value {
                    state = State::AttributeValue;
                } else if ended {
                    // Input ended unexpectedly.
                    store.error(ErrorType::UnexpectedEndOfInput, &iterator);
                    store.clear();
                    break;
                } else {
                    // Because of nested loops, only collect attribute if there is an attribute to collect.
                    if !store.cache.1.is_empty() {
                        // Attribute has no value.
                        store.collect_attribute(QuoteType::None)
                    }
                }
            }
            State::AttributeValue => {
                let mut quote_type = QuoteType::None;
                // Check for quote type.
                match iterator.next() {
                    Some('\'') => quote_type = QuoteType::Single,
                    Some('"') => quote_type = QuoteType::Double,
                    Some(ch) => {
                        if ch == '<' {
                            store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                        }
                        iterator.push(ch);
                        iterator.inner_mut().left();
                    }
                    None => {
                        store.error(ErrorType::UnexpectedEndOfInput, &iterator);
                        break;
                    }
                }
                // Gather values.
                let mut ended = false;
                while !ended {
                    match iterator.next() {
                        Some('\'') if quote_type == QuoteType::Single => break,
                        Some('"') if quote_type == QuoteType::Double => break,
                        Some(ch) if ch.is_whitespace() && quote_type == QuoteType::None => break,
                        // End of tag.
                        Some(ch @ ('>' | '/')) if quote_type == QuoteType::None => {
                            iterator.push(ch);
                            break;
                        }
                        Some(ch) => store.push_attr_value(ch),
                        None => ended = true,
                    }
                }
                if ended {
                    store.error(ErrorType::UnexpectedEndOfInput, &iterator);
                } else {
                    store.collect_attribute(quote_type);
                }

                state = State::AttributeName;
            }
            // A closing tag has been opened.
            State::ClosingTag => match iterator.next() {
                Some(ch) if ch.is_whitespace() => {
                    // tagnames must directly follow the </
                    if store.empty() {
                        store.error(ErrorType::ExpectedTagName, &iterator);
                        state = State::Data;
                    }
                }
                Some(ch) if ch.is_ascii_alphanumeric() || ch == '-' => {
                    // closing tags cannot start with numbers. Reparse the tag as a bogus comment.
                    if store.empty() && ch.is_numeric() {
                        store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                        iterator.push(ch);
                        // Start bogus comment here.
                        store.set_start(&iterator);
                        state = State::BogusComment;
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
                    }
                    state = State::Data
                }
                Some(ch) => {
                    // There was an unexpected character in the closing tag, probably an attribute.
                    store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                    // Skip over the next set of characters until the >.
                    loop {
                        match iterator.next() {
                            Some('>') => {
                                state = State::Data;
                                break;
                            }
                            Some(_) => {}
                            None => break,
                        }
                    }
                }
                None => {
                    store.error(ErrorType::UnexpectedEndOfInput, &iterator);
                }
            },
            // A comment has been opened with <!--
            State::Comment => {
                let mut is_closed = false;
                loop {
                    match iterator.next() {
                        // Attempt to close, First -
                        Some('-') => match iterator.next() {
                            // Second -
                            Some('-') => match iterator.next() {
                                // Hurray. Comment closed.
                                Some('>') => {
                                    is_closed = true;
                                    break;
                                }
                                // Any character, not a >
                                Some(ch) => {
                                    store.push_str("--");
                                    store.push(ch);
                                }
                                None => {
                                    store.push_str("--");
                                    break;
                                }
                            },
                            // Any character, not a -
                            Some(ch) => {
                                store.push('-');
                                store.push(ch);
                            }
                            // Comment unclosed. Retrieve - and break
                            None => {
                                store.push('-');
                                break;
                            }
                        },
                        // Push any character to store.
                        Some(ch) => store.push(ch),
                        None => break,
                    }
                }

                if !is_closed {
                    store.error(ErrorType::UnclosedComment, &iterator);
                }
                store.emit(Event::Comment, &iterator);
                state = State::Data;
            }
            // Result of a parse error. Attempt to parse tag as comment.
            State::BogusComment => {
                loop {
                    match iterator.next() {
                        // Push any character to store.
                        Some('>') | None => break,
                        Some(ch) => store.push(ch),
                    }
                }
                store.emit(Event::Comment, &iterator);
                state = State::Data;
            }
            State::Doctype => {
                let mut ended = false;
                let mut identifier = None;

                // Expect a whitespace character.
                match iterator.next() {
                    Some(ch) => {
                        // Parse error. parse anyway.
                        if !ch.is_whitespace() {
                            iterator.push(ch);
                            store.error(ErrorType::UnexpectedCharacter(ch), &iterator);
                        } else {
                            // One whitespace found, Ignore rest.
                            iterator.next_while(|ch| ch.is_whitespace());
                        }
                    }
                    None => ended = true,
                }

                let root_element: String =
                    iterator.collect_until(|ch| ch.is_whitespace() || ch == &'>');

                // Skip whitespace.
                iterator.next_while(|ch| ch.is_whitespace());

                match iterator.next() {
                    Some('>') => {}
                    // Try to parse identifier.
                    Some(ch @ ('p' | 'P' | 's' | 'S')) => {
                        let mut identifier_string: String = iterator.collect_next(5);
                        identifier_string.insert(0, ch);
                        if identifier_string.to_ascii_lowercase() == "system" {
                            identifier = Some(DocTypeIdentifier::System);
                        } else if identifier_string.to_ascii_lowercase() == "public" {
                            identifier = Some(DocTypeIdentifier::Public);
                        } else {
                            store.error(ErrorType::IndecipherableDocType, &iterator);
                            iterator.find(|ch| ch == &'>');
                        }
                    }
                    Some(_) => {
                        store.error(ErrorType::IndecipherableDocType, &iterator);
                        iterator.find(|ch| ch == &'>');
                    }
                    None => ended = true,
                }
                store.emit(Event::DocType(root_element, identifier), &iterator);
                if ended {
                    store.error(ErrorType::UnexpectedEndOfInput, &iterator);
                    break;
                }
                state = State::Data;
            }
        }
    }

    store.tokens.push(HtmlToken::EOF {
        location: iterator.inner().locus(),
    });

    store
}
