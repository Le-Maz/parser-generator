use regex::Regex;

type TokenMappers<'source, TokenType> = Box<[fn(&'source str) -> TokenType]>;

pub struct Lexer<'source, TokenType> {
    text: &'source str,
    regex: regex::Regex,
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
    fn get_regex() -> String;
    fn get_token_mappers() -> TokenMappers<'source, Self>;
}

impl<'source, TokenType: TokenMetadata<'source>> Lexer<'source, TokenType> {
    pub fn new(text: &'source str) -> Result<Self, regex::Error> {
        Ok(Lexer {
            text,
            regex: Regex::new(&TokenType::get_regex())?,
            token_mappers: TokenType::get_token_mappers(),
            cursor: 0,
        })
    }
}

#[macro_export]
macro_rules! lexer {
    ($vis:vis $token:ident: $($pattern:expr => $name:ident),* $(,)?) => {
        #[derive(Debug, Clone)]
        $vis enum $token<'source> {
            $($name(#[allow(unused)] &'source str)),*
        }

        impl <'source> $crate::lexer::TokenMetadata<'source> for $token<'source> {
            fn get_regex() -> String {
                let mut buffer = String::new();
                for pattern in [$($pattern),*] {
                    buffer.push('(');
                    buffer.push_str(pattern);
                    buffer.push(')');
                    buffer.push('|');
                }
                buffer.pop();
                buffer
            }
            fn get_token_mappers() -> Box<[fn(&'source str) -> $token<'source>]> {
                Box::new([$($token::$name),*])
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::lexer::Lexer;

    crate::lexer! {
        pub Token:
            r"[()]" => Parenthesis,
            r"[a-zA-Z_][a-zA-Z0-9_]*" => Identifier,
            r"[+-]?[0-9][0-9_]*[.][0-9]+" => Float,
            r"[+-]?[0-9][0-9_]*" => Integer,
            r#"".+[^\\]""# => String,
    }

    #[test]
    fn tokenize() {
        let lexer = Lexer::<Token>::new("(123 2_222_333.6 ())").unwrap();
        for token in lexer {
            println!("{token:?}");
        }
    }
}
