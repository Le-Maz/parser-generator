use regex::Regex;

type TokenMappers<'source, TokenType> = &'source [fn(&'source str) -> TokenType];

pub struct Lexer<'source, TokenType> {
    text: &'source str,
    regex: Regex,
    token_mappers: TokenMappers<'source, TokenType>,
    cursor: usize,
}

impl<'source, TokenType> Iterator for Lexer<'source, TokenType> {
    type Item = TokenType;
    fn next(&mut self) -> Option<Self::Item> {
        match self.regex.captures_at(self.text, self.cursor) {
            None => None,
            Some(value) => {
                self.cursor = value.get_match().end();
                let n = value.iter().skip(1).enumerate().find_map(|elem| {
                    elem.1?;
                    Some(elem.0)
                })?;
                let mapper = self.token_mappers[n];
                Some(mapper(value.get_match().as_str()))
            }
        }
    }
}

pub trait TokenMetadata<'source> {
    fn get_regex() -> &'source str;
    fn get_token_mappers() -> TokenMappers<'source, Self>;
}

impl<'source, TokenType: TokenMetadata<'source>> Lexer<'source, TokenType> {
    pub fn new(text: &'source str) -> Result<Self, regex::Error> {
        Ok(Lexer {
            text,
            regex: Regex::new(TokenType::get_regex())?,
            token_mappers: TokenType::get_token_mappers(),
            cursor: 0,
        })
    }
}
