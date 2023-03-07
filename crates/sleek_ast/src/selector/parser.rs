use sleek_utils::QueueIterator;

use crate::AttributeQuoteType as QuoteType;

use super::SelectorStore;

#[derive(Debug)]
pub enum SelectorError {
    MultipleIds,
    InvalidTag,
    EmptySelector,
    InvalidSelector,
}

enum State {
    TagName,
    Class,
    Id,
    AttributeName,
    Start,
    PossibleEnd,
    PossibleNext,
    CompulsoryNext,
    Universal,
    AttributeValue,
}

pub enum Emit {
    Tag,
    Id,
    Class,
    Universal,
    Attribute,
}

pub enum Relation {
    Descendant,
    // Parent,
    Child,
    AdjacentSibling,
    GeneralSibling,
    Group,
}

/// Parse an input string into a `SelectorStore`.
///
/// Returns a `SelectorError` instead if there is a error encountered while parsing.
pub fn parse_selector(selector: &str) -> Result<SelectorStore, SelectorError> {
    let mut chars = QueueIterator::new(selector.chars());
    let mut store = SelectorStore::new();
    let mut state = State::Start;

    loop {
        match state {
            // Initial or rest State.
            State::Start => match chars.next() {
                Some('\t' | '\n' | '\x0C' | ' ' | '\r') => {}
                Some('.') => state = State::Class,
                Some('*') => state = State::Universal,
                Some('#') => state = State::Id,
                Some('[') => state = State::AttributeName,
                Some(ch @ ('A'..='Z' | 'a'..='z' | '_' | '-')) => {
                    store.collect(ch);
                    state = State::TagName;
                }
                Some('0'..='9') => Err(SelectorError::InvalidTag)?,
                Some(_) => Err(SelectorError::InvalidSelector)?,
                None => Err(SelectorError::EmptySelector)?,
            },
            // Parsing a class. After a .
            State::Class => match chars.next() {
                Some('\t' | '\n' | '\x0C' | ' ' | '\r') => {
                    store.emit(Emit::Class)?;
                    state = State::PossibleNext;
                }
                Some(ch @ ('[' | '.' | ':' | '#' | ',')) => {
                    // Push parsed data.
                    store.emit(Emit::Class)?;
                    state = State::Start;
                    chars.push(ch);
                }
                Some(ch) => store.collect(ch),
                None => {
                    store.emit(Emit::Class)?;
                    break;
                }
            },
            // After a * pattern.
            State::Universal => {
                store.has_data = true;
                store.emit(Emit::Universal)?;
                match chars.next() {
                    Some('\t' | '\n' | '\x0C' | ' ' | '\r') => {
                        state = State::PossibleNext;
                    }
                    Some(ch @ ('[' | '.' | ':' | '#' | ',')) => {
                        state = State::Start;
                        chars.push(ch);
                    }
                    Some(ch @ ('>' | '+' | '~')) => {
                        state = State::PossibleNext;
                        chars.push(ch);
                    }
                    Some(_) => Err(SelectorError::InvalidSelector)?,
                    None => break,
                }
            }

            // Expecting an id. After a #.
            State::Id => match chars.next() {
                Some('\t' | '\n' | '\x0C' | ' ' | '\r') => {
                    store.emit(Emit::Id)?;
                    state = State::PossibleNext;
                }
                Some(ch @ ('[' | '.' | ':' | '#' | ',')) => {
                    // Push parsed data.
                    store.emit(Emit::Id)?;
                    state = State::Start;
                    chars.push(ch);
                }
                Some(ch) => store.collect(ch),
                None => {
                    store.emit(Emit::Id)?;
                    break;
                }
            },
            // Expecting an attribute name. After a [
            State::AttributeName => match chars.next() {
                Some('\t' | '\n' | '\x0C' | ' ' | '\r' | '[' | '.' | ':' | '#' | ',') | None => {
                    Err(SelectorError::InvalidSelector)?
                }
                Some('=') => state = State::AttributeValue,
                Some(']') => {
                    store.emit(Emit::Attribute)?;
                    state = State::PossibleEnd;
                }
                Some(ch) => store.collect(ch),
            },

            // Expecting an attribute value. After a =
            State::AttributeValue => {
                let mut quote_type = QuoteType::None;

                match chars.next() {
                    Some('\'') => quote_type = QuoteType::Single,
                    Some('"') => quote_type = QuoteType::Double,
                    Some(ch) => {
                        if ch == '<' {
                            Err(SelectorError::InvalidSelector)?
                        }
                        chars.push(ch);
                    }
                    None => Err(SelectorError::InvalidSelector)?,
                }

                loop {
                    match (&quote_type, chars.next()) {
                        (QuoteType::Single, Some('\'')) | (QuoteType::Double, Some('"')) => break,
                        (_, Some(ch)) => {
                            if quote_type == QuoteType::None {
                                if ch == ']' {
                                    chars.push(ch);
                                } else if matches!(ch, '>' | '/') || ch.is_whitespace() {
                                    Err(SelectorError::InvalidSelector)?
                                }
                            }
                            store.collect_2(ch);
                        }
                        (_, None) => Err(SelectorError::InvalidSelector)?,
                    }
                }

                chars.next_while(|ch| ch.is_whitespace());

                match chars.next() {
                    Some(']') => store.emit(Emit::Attribute)?,
                    _ => Err(SelectorError::InvalidSelector)?,
                }
                state = State::PossibleEnd;
            }
            // Parsing a tagname. After the first character in the name.
            State::TagName => match chars.next() {
                Some('\t' | '\n' | '\x0C' | ' ' | '\r') => {
                    store.emit(Emit::Tag)?;
                    state = State::PossibleNext;
                }
                Some(ch @ ('[' | '.' | ':' | '#' | ',')) => {
                    // Push parsed data.
                    store.emit(Emit::Tag)?;
                    state = State::Start;
                    chars.push(ch);
                }
                Some(ch @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-')) => store.collect(ch),
                Some(_) => Err(SelectorError::InvalidSelector)?,
                None => {
                    store.emit(Emit::Tag)?;
                    break;
                }
            },

            // Expecting the end of input.
            State::PossibleEnd => match chars.next() {
                Some(ch @ ('[' | '.' | ':' | '#' | ',')) => {
                    state = State::Start;
                    chars.push(ch);
                }
                Some(ch @ ('\t' | '\n' | '\x0C' | ' ' | '\r')) => {
                    state = State::PossibleNext;
                    chars.push(ch);
                }
                Some(_) => todo!(),
                None => break,
            },

            // Expecting a possible decendant selector.
            State::PossibleNext => match chars.next() {
                Some('\t' | '\n' | '\x0C' | ' ' | '\r') => {
                    chars.next_while(|ch| ch.is_whitespace())
                }
                Some('>') => {
                    store.shift(Relation::Child);
                    state = State::CompulsoryNext;
                }
                Some(ch) => {
                    store.shift(Relation::Descendant);
                    state = State::Start;
                    chars.push(ch);
                }
                None => break,
            },

            // Expecting a child, sibling or parent selector.
            State::CompulsoryNext => match chars.next() {
                Some('\t' | '\n' | '\x0C' | ' ' | '\r') => {}
                Some(ch) => {
                    state = State::Start;
                    chars.push(ch);
                }
                None => Err(SelectorError::InvalidSelector)?,
            },
        }
    }
    Ok(store)
}
