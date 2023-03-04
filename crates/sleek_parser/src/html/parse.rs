use std::path::Path;

use crate::HtmlParseError;

use super::tokenize::tokenize_html;

/// The result of the Html parsing process.
/// The parser tries to produce a valid HTML DOM Tree regardless of how wrangled or broken the input string is.
/// The resulting tree, as well as errors encountered, are stored on this struct.
pub struct HtmlParseResult {
    pub tree: sleek_ast::HtmlTree,
    pub errors: Vec<HtmlParseError>,
}

/// Parse an HTML string into a valid DOM tree.
pub fn parse_html_input(input: &str) -> HtmlParseResult {
    let mut parse_errors = vec![];
    let mut tokenizer_result = tokenize_html(input);
    parse_errors.append(&mut tokenizer_result.errors);
    todo!()
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
