mod error;
mod parse;
mod test;
mod tokenize;

use std::path::Path;

pub use error::HtmlParseError;

use tokenize::tokenize_html;

use parse::HtmlParser;

pub use self::parse::HtmlParseResult;

/// Parse an HTML string into a valid DOM tree.
pub fn parse_html_input(input: &str) -> HtmlParseResult {
    HtmlParser::parse(tokenize_html(input))
}

/// Parse an HTML file into a valid DOM tree.
/// # Errors
/// The function will return an error if:
/// - The file does not exist.
/// - The file is not a valid HTML file.
/// - There was an error encountered while reading the file.
/// - There are not adequate permissions.
pub fn parse_html_file<P: AsRef<Path>>(path: P) -> std::io::Result<HtmlParseResult> {
    let address = path.as_ref().to_str().unwrap();
    match address.split('.').last() {
        Some("html") | Some("htm") | Some("xhtml") | Some("dhtml") => {
            let input = std::fs::read_to_string(path)?;
            Ok(parse_html_input(&input))
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
