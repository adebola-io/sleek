pub enum HtmlParseErrorType {
    InvalidCharacter,
    UnexpextedEndOfFile,
    UnexpectedCharacter,
    ExpectedTagName,
}

pub struct HtmlParseError {
    pub error_type: HtmlParseErrorType,
    pub location: [usize; 2],
}
