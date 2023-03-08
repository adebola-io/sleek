use sleek_ast::HtmlDocument;

use crate::HtmlParseError;

mod speculative;
mod synchronous;

pub use speculative::{ParserResponse, SpeculativeHtmlParser};
pub use synchronous::SyncHtmlParser;

/// The result of the Html parsing process.
/// The parser tries to produce a valid HTML DOM Tree regardless of how wrangled or broken the input string is.
/// The resulting tree, as well as errors encountered, are stored on this struct.
#[derive(Debug)]
pub struct HtmlParseResult {
    pub tree: HtmlDocument,
    pub errors: Vec<HtmlParseError>,
}
