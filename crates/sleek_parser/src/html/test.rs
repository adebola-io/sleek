#[cfg(test)]
mod tests {

    use std::{mem::take, time::Instant};

    use crate::{
        html::{
            tokenizer::{tokenize, TokenStore},
            ParseMode,
        },
        parse_html_input, HtmlParseError, HtmlParseResult,
    };
    use sleek_ast::{HtmlTag, HtmlToken, Query};
    use sleek_utils::{MatrixIterator, Node, QueueIterator};

    pub struct TokenizerResult {
        errors: Vec<HtmlParseError>,
        tokens: Vec<HtmlToken>,
    }

    fn tokenize_html(input: &str) -> TokenizerResult {
        let (mut token_store, mut iterator) = (
            TokenStore::new(),
            QueueIterator::new(MatrixIterator::new(input.chars(), '\n')),
        );
        tokenize(&mut token_store, &mut iterator);
        TokenizerResult {
            errors: take(&mut token_store.errors),
            tokens: take(&mut token_store.tokens),
        }
    }

    #[test]
    fn it_tokenizes_plain_html() {
        let res = tokenize_html("This is an example of plain text in Html.");
        assert_eq!(res.tokens.len(), 2, "Tokenized {:?}", res.tokens);
        assert_eq!(res.errors.len(), 0);
    }
    #[test]
    fn it_tokenizes_unclosed_tag() {
        let res = tokenize_html("<");
        assert_eq!(res.tokens.len(), 2);
        assert_eq!(res.errors.len(), 1);
    }

    #[test]
    fn it_tokenizes_tag_name() {
        let res = tokenize_html("<html$$$$></html$$$$>");
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 3, "Tokenized {:?}", res.tokens);
    }

    #[test]
    fn it_tokenizes_open_tag() {
        let res = tokenize_html("<tag>");
        assert!(matches!(res.tokens[0], HtmlToken::OpeningTag { .. }));
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 2, "Tokenized {:?}", res.tokens);
    }
    #[test]
    fn it_tokenizes_self_closing_open_tag() {
        let res = tokenize_html("<input />");
        assert!(matches!(res.tokens[0], HtmlToken::OpeningTag { .. }));
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 2, "Tokenized {:?}", res.tokens);
    }
    #[test]
    fn it_tokenizes_open_tag_with_simple_attribute() {
        let res = tokenize_html("<button disabled/>");
        assert!(matches!(
            &res.tokens[0],
            HtmlToken::OpeningTag { attributes, .. } if attributes.len() == 1
        ));
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 2, "Tokenized {:?}", res.tokens);
    }
    #[test]
    fn it_tokenizes_open_tag_with_multiple_simple_attributes() {
        let res = tokenize_html(
            "<button
            disabled
            attrib
            labelled   />",
        );
        assert!(matches!(
            &res.tokens[0],
            HtmlToken::OpeningTag { attributes, .. } if attributes.len() == 3
        ));
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 2, "Tokenized {:?}", res.tokens);
    }
    #[test]
    fn it_tokenizes_open_tag_with_single_quote_attribute() {
        let res = tokenize_html("<input type='text'/>");
        assert!(matches!(
            &res.tokens[0],
            HtmlToken::OpeningTag
            { name: HtmlTag::Input, attributes, .. }
            if attributes.len() == 1 && attributes[0].key == "type"
        ));
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 2, "Tokenized {:?}", res.tokens);
    }
    #[test]
    fn it_tokenizes_multiple_attribute_types() {
        let res = tokenize_html("<div class=\"box\" ref=mutable id='box-1' hello></div>");
        assert!(matches!(
            &res.tokens[0],
            HtmlToken::OpeningTag
            { name: HtmlTag::Div, attributes, .. } if attributes.len() == 4
        ));
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 3, "Tokenized {:?}", res.tokens);
    }
    #[test]
    fn it_rejects_fragment_tag() {
        let res = tokenize_html("<></>");
        assert_eq!(res.errors.len(), 2, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 2);
    }
    #[test]
    fn it_tokenizes_close_tag() {
        let result = tokenize_html("</html>");
        assert_eq!(result.tokens.len(), 2, "Tokenized {:?}", result.tokens);
        assert_eq!(result.errors.len(), 0);
    }
    #[test]
    fn it_rejects_numeric_tags() {
        let result = tokenize_html("<123></123>");
        assert_eq!(result.errors.len(), 2, "Errors: {:?}", result.errors);
        assert_eq!(result.tokens.len(), 3, "Tokenized {:?}", result.tokens);
    }
    #[test]
    fn it_tokenizes_full_element() {
        let res = tokenize_html("<p>This is a complete paragraph element.</p>");
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 4, "Tokenized {:?}", res.tokens);
    }
    #[test]
    fn it_tokenizes_less_than_and_exclamation_as_comment() {
        let res = tokenize_html("<!");
        assert_eq!(res.errors.len(), 1, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 2, "Tokenized {:?}", res.tokens);
    }
    #[test]
    fn it_tokenizes_unclosed_comment() {
        let res = tokenize_html("<!- This is an unclosed comment.");
        assert_eq!(res.errors.len(), 2, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 2, "Tokenized {:?}", res.tokens);
    }
    #[test]
    fn it_tokenizes_element_with_comment() {
        let res = tokenize_html(
            "
            <html>
                <head>
                <title>Document</title>
                <!-- This is a comment -->
                </head>
            </html>
            ",
        );
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 9, "Tokenized {:?}", res.tokens);
    }

    #[test]
    fn it_tokenizes_html_fragment() {
        let res = tokenize_html(
            "
    <!DOCTYPE html>
    <html lang=\"en\">
    <head>
       <meta charset=\"UTF-8\"/>
       <meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge\">
       <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
       <title>Document</title>
    </head>
    <body>
       <!-- This is a comment in Html. -->
    </body>
    </html>",
        );
        assert_eq!(res.errors.len(), 0, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 15, "Tokenized {:?}", res.tokens);
    }

    #[test]
    fn it_tokenizes_html_fragment_2() {
        let tokenizer_result = tokenize_html(
                "
        <div class=\"head\" id=\"head\">
        <p>
            <a href=\"http://www.w3.org/\">
                <img alt=\"W3C\" height=\"48\" src=\"http://www.w3.org/Icons/w3c_home\" width=\"72\">
            </a>
        </p>
        <h1>HTML5</h1>
       </div>",
            );

        assert!(tokenizer_result.tokens.len() != 0);
    }

    #[test]
    fn it_tokenizes_broken_html() {
        let res = tokenize_html(
            "
    <html lang=\"en\">
       <!head>
    </html>
    ",
        );
        assert_eq!(res.errors.len(), 1, "Errors: {:?}", res.errors);
        assert_eq!(res.tokens.len(), 4, "Tokenized {:?}", res.tokens);
    }

    #[test]
    fn it_parses_html() {
        let input = "<html lang=en>This is valid html.</html>";
        let HtmlParseResult { tree, errors } = parse_html_input(input, ParseMode::Synchronous);
        assert_eq!(tree.query_selector("html"), tree.children().next().cloned());
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn it_parses_html_with_children() {
        let HtmlParseResult { tree, errors } = parse_html_input(
            "<html lang=\"en\">
                    <head>
                        <title>Document</title>
                    </head>
                    <body>
                        <section>
                            <h1>Hello, World!</h1>
                            <p>This is a document section.</p>
                        </section>
                    </body>
                </html>",
            ParseMode::Synchronous,
        );

        assert_eq!(tree.nodes.len(), 1);

        assert_eq!(
            tree.query_selector("title").unwrap().get_text_content(),
            "Document"
        );
        assert_eq!(errors.len(), 0, "Errors encountered: {:?}", errors);
    }

    #[test]
    fn it_parses_file() {
        let input = std::fs::read_to_string("src/html/test.html").unwrap();
        let time = Instant::now();
        let res = parse_html_input(&input, ParseMode::Synchronous);
        println!("{}", res.tree.query_selector_all("div[class]").len());
        println!("{:?}", time.elapsed());
        assert_eq!(res.errors.len(), 0)
    }

    #[test]
    fn it_parses_elements_speculatively() {
        let res = parse_html_input(
            "<div><input/><div><span></span></div></div>",
            ParseMode::Speculative,
        );
        println!("{:#?}", res.tree);

        assert_eq!(res.errors.len(), 0, "Errors encountered: {:?}", res.errors)
    }

    #[test]
    fn it_parses_elements_with_text_speculatively() {
        let res = parse_html_input(
            "<html>
                <head></head>
                <body>
                    <p>This is inner text.</p>
                </body>
            </html>",
            ParseMode::Speculative,
        );
        assert_eq!(
            res.tree.children().next().unwrap().get_text_content(),
            "This is inner text."
        );

        assert_eq!(res.errors.len(), 0, "Errors encountered: {:?}", res.errors)
    }

    #[test]
    fn it_finds_unclosed_tags_speculatively() {
        let res = parse_html_input(
            "<div>
        <button type=button>
        <p> Hello world! </p>
        </div>",
            ParseMode::Speculative,
        );
        assert_eq!(res.errors.len(), 2, "Errors encountered: {:?}", res.errors)
    }
}
