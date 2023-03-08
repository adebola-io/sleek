mod error;
mod parser;
mod test;
mod tokenizer;

use std::path::Path;

pub use error::*;

pub use self::parser::HtmlParseResult;
use self::{
    parser::{SpeculativeHtmlParser, SyncHtmlParser},
    tokenizer::TokenStore,
};
use sleek_utils::{MatrixIterator, QueueIterator};

#[allow(dead_code)]
pub enum ParseMode {
    /// Build the tree as each token is parsed. Allows easier handling of internal styles and scripts.
    Speculative,
    /// Tokenize all at once before sending to the parser.
    Synchronous,
}

/// Parse an HTML string into a valid DOM tree.
pub fn parse_html_input(input: &str, mode: ParseMode) -> HtmlParseResult {
    let iterator = QueueIterator::new(MatrixIterator::new(input.chars(), '\n'));
    let token_store = TokenStore::new();

    match mode {
        ParseMode::Speculative => SpeculativeHtmlParser::parse(token_store, iterator),
        ParseMode::Synchronous => SyncHtmlParser::parse(token_store, iterator),
    }
}

/// Parse an HTML file into a valid DOM tree.
/// # Errors
/// The function will return an error if:
/// - The file does not exist.
/// - The file is not a valid HTML file.
/// - There was an error encountered while reading the file.
/// - There are not adequate permissions.
pub fn parse_html_file<P: AsRef<Path>>(
    path: P,
    mode: ParseMode,
) -> std::io::Result<HtmlParseResult> {
    let address = path.as_ref().to_str().unwrap();
    match address.split('.').last() {
        Some("html") | Some("htm") | Some("xhtml") | Some("dhtml") => {
            let input = std::fs::read_to_string(path)?;
            Ok(parse_html_input(&input, mode))
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidFilename,
            "Could not read file. Expected valid html file as input",
        )),
        Some(ext) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidFilename,
            format!("\"{ext}\" is not a supported file extension for html"),
        )),
    }
}
