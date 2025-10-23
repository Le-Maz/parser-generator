use std::{
    collections::HashMap,
    sync::{LazyLock, nonpoison::Mutex},
};

use regex::Regex;

type TokenMappers<'source, TokenType> = &'source [fn(&'source str) -> TokenType];

pub struct Lexer<'source, TokenType> {
    text: &'source str,
    regex: &'static Regex,
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
    fn get_regex() -> &'static str;
    fn get_token_mappers() -> TokenMappers<'source, Self>;
}

static CACHED_REGEXES: LazyLock<
    Mutex<HashMap<&'static str, &'static Result<Regex, regex::Error>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

impl<'source, TokenType: TokenMetadata<'source>> Lexer<'source, TokenType> {
    fn cached_regex() -> &'static Result<Regex, regex::Error> {
        let regex = TokenType::get_regex();
        CACHED_REGEXES
            .lock()
            .entry(regex)
            .or_insert_with(|| Box::leak(Box::new(Regex::new(regex))))
    }
    pub fn new(text: &'source str) -> Result<Self, &'static regex::Error> {
        let regex = match Self::cached_regex() {
            Ok(regex) => regex,
            Err(error) => return Err(error),
        };
        Ok(Lexer {
            text,
            regex,
            token_mappers: TokenType::get_token_mappers(),
            cursor: 0,
        })
    }
}
