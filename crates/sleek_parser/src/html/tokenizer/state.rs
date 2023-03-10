use std::str::Chars;

use sleek_utils::{HigherOrderIterator, QueueMatrix};

use sleek_ast::{AttributeQuoteType as QuoteType, DocTypeIdentifier, HtmlToken};

use super::store::{Event, TokenStore};
use crate::html::HtmlParseErrorType as ErrorType;

#[derive(Debug)]
pub enum State {
    Data,
    OpeningTag,
    ClosingTag,
    BogusComment,
    AttributeName,
    Comment,
    AttributeValue,
    Doctype,
}

/// Tokenize an input string.
pub fn tokenize(token_store: &mut TokenStore, iterator: &mut QueueMatrix<Chars<'_>>) {
    // Starting state.
    let mut state = State::Data;

    loop {
        match state {
            // Parse regular html text, without any formatting.
            State::Data => match iterator.next() {
                Some('<') => {
                    if !token_store.empty() {
                        token_store.emit(Event::Text, iterator);
                    }
                    token_store.set_start(iterator);
                    state = State::OpeningTag
                }
                // Parse error caused by null character.
                Some('\0') => {
                    if !token_store.empty() {
                        token_store.emit(Event::Text, iterator);
                    }
                    token_store.error(ErrorType::InvalidCharacter, iterator);
                }
                Some(ch) => {
                    // Collect the starting point of the text node.
                    if token_store.empty() {
                        token_store.set_start(iterator)
                    }
                    token_store.push(ch)
                }
                None => {
                    if !token_store.empty() {
                        token_store.emit(Event::Text, iterator);
                    }
                    break;
                }
            },
            // A tag has been opened.
            State::OpeningTag => match iterator.next() {
                Some('!') => {
                    if token_store.empty() {
                        // Comment or Doctype tag.
                        // Collect next two characters and check if they match commment start (--).
                        match iterator.next() {
                            // First -
                            Some('-') => match iterator.next() {
                                // Second -
                                Some('-') => state = State::Comment,
                                // Any other character. Take first - as part of the comment.
                                Some(ch) => {
                                    token_store.push('-');
                                    token_store.push(ch);
                                    token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                                    state = State::Comment
                                }
                                None => {
                                    token_store.push('-');
                                    token_store.emit(Event::Comment, iterator);
                                    token_store.error(ErrorType::UnexpectedEndOfInput, iterator);
                                    break;
                                }
                            },
                            // Default to comment.
                            None => {
                                token_store.emit(Event::Comment, iterator);
                                token_store.error(ErrorType::UnexpectedEndOfInput, iterator);
                                break;
                            }
                            // Check for !doctype
                            Some(ch @ ('d' | 'D')) => {
                                let value: String = iterator.collect_next(6);
                                if value.to_ascii_lowercase() == "octype" {
                                    state = State::Doctype;
                                } else {
                                    println!("{value}");
                                    token_store.push(ch);
                                    token_store.push_str(value.as_str());
                                    token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                                    state = State::Comment;
                                }
                            }
                            Some(ch) => {
                                token_store.push(ch);
                                token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                                state = State::BogusComment;
                            }
                        }
                    } else {
                        // Tagnames has already started parsing.
                        token_store.push('!');
                    }
                }
                Some('/') => {
                    // There is nothing between < and /. Parse as closing tag.
                    if token_store.empty() {
                        state = State::ClosingTag
                    } else {
                        // Open tag is possibly self-closing.
                        iterator.next_while(|ch| ch.is_whitespace());
                        match iterator.next() {
                            // tag is self-closing.
                            Some('>') => {
                                token_store.emit(Event::OpenerTag(true), iterator);
                                state = State::Data;
                            }
                            // Parse error. Scan character again as attribute.
                            Some(ch) => {
                                token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                                iterator.push(ch);
                                state = State::AttributeName;
                            }
                            // Tag was unclosed.
                            None => {
                                token_store.clear();
                                token_store.error(ErrorType::UnexpectedEndOfInput, iterator);
                                state = State::Data;
                            }
                        }
                    }
                }
                Some('>') => {
                    // Parse <> as text.
                    if token_store.empty() {
                        token_store.push('<');
                        iterator.push('>');
                        token_store.error(ErrorType::UnexpectedCharacter('>'), iterator);
                        state = State::Data;
                    } else {
                        // Push an opening tag with no attributes.
                        token_store.emit(Event::OpenerTag(false), iterator);
                        state = State::Data;
                    }
                }
                Some(ch) if ch.is_ascii_alphanumeric() || ch == '-' => {
                    // Tags cannot start with numeric values. Reparse the tag as plain text.
                    if token_store.empty() && (ch.is_numeric() || ch == '-') {
                        token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                        token_store.push('<');
                        token_store.push(ch);
                        state = State::Data;
                    } else {
                        token_store.push(ch.to_ascii_lowercase());
                    }
                }
                Some(ch) if ch.is_whitespace() => {
                    // Parse error. Expected a tagname to be present. Reparse tag as text.
                    if token_store.empty() {
                        token_store.push('<');
                        iterator.push(ch);
                        token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                        state = State::Data;
                    } else {
                        // Parsing an attribute. Revisit the current character as an attribute name.
                        iterator.push(ch);
                        state = State::AttributeName;
                    }
                }
                Some(ch) => {
                    if token_store.empty() {
                        // Emit as text.
                        iterator.push('<');
                        iterator.push(ch);
                        token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                        state = State::Data;
                    } else {
                        // Add to tag name.
                        token_store.push(ch);
                    }
                }
                None => {
                    token_store.error(ErrorType::UnexpectedEndOfInput, iterator);
                    // Emit as text.
                    token_store.push('<');
                    token_store.emit(Event::Text, iterator);
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
                        Some(ch) => token_store.push_attr_name(ch),
                        // Input ends abruptly.
                        None => ended = true,
                    }
                }
                // = encountered. attribute equals something.
                if has_value {
                    state = State::AttributeValue;
                } else if ended {
                    // Input ended unexpectedly.
                    token_store.error(ErrorType::UnexpectedEndOfInput, iterator);
                    token_store.clear();
                    break;
                } else {
                    // Because of nested loops, only collect attribute if there is an attribute to collect.
                    if !token_store.cache.1.is_empty() {
                        // Attribute has no value.
                        token_store.collect_attribute(QuoteType::None)
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
                            token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                        }
                        iterator.push(ch);
                        iterator.inner_mut().left();
                    }
                    None => {
                        token_store.error(ErrorType::UnexpectedEndOfInput, iterator);
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
                        Some(ch) => token_store.push_attr_value(ch),
                        None => ended = true,
                    }
                }
                if ended {
                    token_store.error(ErrorType::UnexpectedEndOfInput, iterator);
                } else {
                    token_store.collect_attribute(quote_type);
                }

                state = State::AttributeName;
            }
            // A closing tag has been opened.
            State::ClosingTag => match iterator.next() {
                Some(ch) if ch.is_whitespace() => {
                    // tagnames must directly follow the </
                    if token_store.empty() {
                        token_store.error(ErrorType::ExpectedTagName, iterator);
                        state = State::Data;
                    }
                }
                Some(ch) if ch.is_ascii_alphanumeric() || ch == '-' => {
                    // closing tags cannot start with numbers. Reparse the tag as a bogus comment.
                    if token_store.empty() && ch.is_numeric() {
                        token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                        iterator.push(ch);
                        // Start bogus comment here.
                        token_store.set_start(iterator);
                        state = State::BogusComment;
                    } else {
                        // Collect character for html tagname.
                        token_store.push(ch.to_ascii_lowercase());
                    }
                }
                Some('>') => {
                    // Tag was closed without any name.
                    if token_store.empty() {
                        token_store.error(ErrorType::ExpectedTagName, iterator);
                    } else {
                        token_store.emit(Event::Close, iterator);
                    }
                    state = State::Data
                }
                Some(ch) => {
                    if token_store.empty() {
                        // Emit as text.
                        iterator.push(ch);
                        token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                        state = State::BogusComment
                    } else {
                        // Add to tag name.
                        token_store.push(ch);
                    }
                }
                None => {
                    token_store.error(ErrorType::UnexpectedEndOfInput, iterator);
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
                                    token_store.push_str("--");
                                    token_store.push(ch);
                                }
                                None => {
                                    token_store.push_str("--");
                                    break;
                                }
                            },
                            // Any character, not a -
                            Some(ch) => {
                                token_store.push('-');
                                token_store.push(ch);
                            }
                            // Comment unclosed. Retrieve - and break
                            None => {
                                token_store.push('-');
                                break;
                            }
                        },
                        // Push any character to store.
                        Some(ch) => token_store.push(ch),
                        None => break,
                    }
                }

                if !is_closed {
                    token_store.error(ErrorType::UnclosedComment, iterator);
                }
                token_store.emit(Event::Comment, iterator);
                state = State::Data;
            }
            // Result of a parse error. Attempt to parse tag as comment.
            State::BogusComment => {
                loop {
                    match iterator.next() {
                        // Push any character to store.
                        Some('>') | None => break,
                        Some(ch) => token_store.push(ch),
                    }
                }
                token_store.emit(Event::Comment, iterator);
                state = State::Data;
            }
            State::Doctype => {
                let mut _force_quirks = false;
                let mut ended = false;
                let mut r#type = None;

                // Expect a whitespace character.
                match iterator.next() {
                    Some(ch) => {
                        // Parse error. parse anyway.
                        if ch.is_whitespace() {
                            // One whitespace found, Ignore rest.
                            iterator.next_while(|ch| ch.is_whitespace());
                        } else {
                            iterator.push(ch);
                            token_store.error(ErrorType::UnexpectedCharacter(ch), iterator);
                        }
                    }
                    None => {
                        ended = true;
                    }
                }

                let name: String = iterator.collect_until(|ch| ch.is_whitespace() || ch == &'>');

                // Skip whitespace.
                iterator.next_while(|ch| ch.is_whitespace());

                match iterator.next() {
                    Some('>') => {
                        _force_quirks = true;
                        token_store.error(ErrorType::UnexpectedCharacter('>'), iterator);
                    }
                    Some('\0') => {
                        token_store.error(ErrorType::InvalidCharacter, iterator);
                        token_store.push('\u{fffd}');
                    }
                    // Try to parse identifier.
                    Some(ch @ ('p' | 'P' | 's' | 'S')) => {
                        let mut identifier_string: String = iterator.collect_next(5);
                        identifier_string.insert(0, ch);
                        if identifier_string.to_ascii_lowercase() == "system" {
                            r#type = Some(DocTypeIdentifier::System);
                        } else if identifier_string.to_ascii_lowercase() == "public" {
                            r#type = Some(DocTypeIdentifier::Public);
                        } else {
                            token_store.error(ErrorType::IndecipherableDocType, iterator);
                            iterator.find(|ch| ch == &'>');
                        }
                    }
                    Some(_) => {
                        token_store.error(ErrorType::IndecipherableDocType, iterator);
                        iterator.find(|ch| ch == &'>');
                    }
                    None => ended = true,
                }
                if ended {
                    _force_quirks = true;
                    token_store.error(ErrorType::UnexpectedEndOfInput, iterator);
                    break;
                }
                token_store.emit(
                    Event::DocType {
                        name,
                        r#type,
                        force_quirks: _force_quirks,
                    },
                    iterator,
                );
                state = State::Data;
            }
        }
    }

    token_store.tokens.push(HtmlToken::EOF {
        location: iterator.inner().locus(),
    });
}
