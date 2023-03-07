use sleek_ast::HtmlTag;

#[derive(Debug)]
pub enum HtmlParseErrorType {
    InvalidCharacter,
    UnexpectedEndOfInput,
    UnexpectedCharacter(char),
    ExpectedTagName,
    UnclosedComment,
    IndecipherableDocType,
    SelfClosingNonVoidTag,
    VoidElementEndTag(HtmlTag),
    UnclosedTag(HtmlTag),
    UnexpectedCloseTag(HtmlTag),
}

#[derive(Debug)]
pub struct HtmlParseError {
    pub error_type: HtmlParseErrorType,
    pub location: [usize; 2],
}
