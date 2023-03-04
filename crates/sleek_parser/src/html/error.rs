#[derive(Debug)]
pub enum HtmlParseErrorType {
    InvalidCharacter,
    UnexpectedEndOfInput,
    UnexpectedCharacter(char),
    ExpectedTagName,
    UnclosedComment,
    IndecipherableDocType,
}

#[derive(Debug)]
pub struct HtmlParseError {
    pub error_type: HtmlParseErrorType,
    pub location: [usize; 2],
}
