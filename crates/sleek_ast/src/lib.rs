#![feature(let_chains)]
#![allow(incomplete_features)]
#![feature(return_position_impl_trait_in_trait)]

mod element;
mod event;
mod html_node;
mod query;
mod selector;
mod tag;
mod tests;
mod token;
mod tree;

pub use element::{AttributeData, ElementRef};
pub use event::*;
pub use html_node::*;
pub use query::Query;
pub use selector::*;
pub use tag::HtmlTag;
pub use token::{AttributeQuoteType, DocTypeIdentifier, HtmlAttribute, HtmlToken};
pub use tree::HtmlDocument;

// Parsing
