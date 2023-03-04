#![feature(io_error_more)]
mod html;

pub use html::{
    parse::{parse_html_file, parse_html_input, HtmlParseResult},
    HtmlParseError,
};
