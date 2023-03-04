#[cfg(test)]
mod tests {
    use crate::html::tokenize::tokenize_html;

    #[test]
    fn it_tokenizes_plain_html() {
        let tokenizer_result = tokenize_html("This is an example of plain text in Html.");
        println!("{:?}", tokenizer_result.tokens[0]);
        assert_eq!(tokenizer_result.tokens.len(), 1);
        assert_eq!(tokenizer_result.errors.len(), 0);
    }
    #[test]
    fn it_tokenizes_unclosed_tag() {
        let tokenizer_result = tokenize_html("<");
        println!("{:?}", tokenizer_result.tokens[0]);
        assert_eq!(tokenizer_result.tokens.len(), 1);
        assert_eq!(tokenizer_result.errors.len(), 1);
    }
    #[test]
    fn it_tokenizes_close_tag() {
        let result = tokenize_html("</html>");
        println!("{:?}", result.tokens);
        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.errors.len(), 0);
    }
}
