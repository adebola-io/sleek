use sleek_ast::HtmlTag;

#[derive(Debug, Default)]
pub enum HtmlParseErrorType {
    #[default]
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

#[derive(Debug, Default)]
pub struct HtmlParseError {
    pub error_type: HtmlParseErrorType,
    pub location: [usize; 2],
}
