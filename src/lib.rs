pub use regex;

pub mod lexer;
pub mod parser;

#[macro_export]
macro_rules! parser {
    (
        $token_vis:vis $token_enum:ident:
            $($token_variant:ident = $token_regex:literal),*;
        $symbol_vis:vis $symbol_enum:ident:
            $($symbol:ident = $($symbol_case:ident($($part:ident $part_binding:ident)*))|*),*;
    ) => {
        #[allow(unused)]
        #[derive(Debug, Clone)]
        $token_vis enum $token_enum<'source> {
            $($token_variant(&'source str)),*
        }

        impl <'source> $crate::lexer::TokenMetadata<'source> for $token_enum<'source> {
            fn get_regex() -> String {
                let mut buffer = String::new();
                for pattern in [$($token_regex),*] {
                    buffer.push('(');
                    buffer.push_str(pattern);
                    buffer.push(')');
                    buffer.push('|');
                }
                buffer.pop();
                buffer
            }
            fn get_token_mappers() -> Box<[fn(&'source str) -> $token_enum<'source>]> {
                Box::new([$($token_enum::$token_variant),*])
            }
        }
        $(
            #[allow(unused)]
            #[derive(Debug, Clone)]
            $symbol_vis struct $token_variant<'tokens>(&'tokens str);
        )*
        $(
            #[derive(Debug, Clone)]
            $symbol_vis enum $symbol<'tokens> {
                $($symbol_case{$($part_binding: ::std::rc::Rc<$part<'tokens>>),*}),*,
            }
        )*
        #[derive(Debug, Clone)]
        $symbol_vis enum $symbol_enum<'tokens> {
            $(
                $token_variant(::std::rc::Rc<$token_variant<'tokens>>),
            )*
            $(
                $symbol(::std::rc::Rc<$symbol<'tokens>>),
            )*
        }
        impl <'tokens> From<$token_enum<'tokens>> for $symbol_enum<'tokens> {
            fn from(token: $token_enum<'tokens>) -> Self {
                match token {
                    $($token_enum::$token_variant(token) => $symbol_enum::$token_variant(::std::rc::Rc::new($token_variant(token))),)*
                    #[allow(unreachable_patterns)] _ => unimplemented!(),
                }
            }
        }
        impl <'tokens> $crate::parser::SymbolMetadata for $symbol_enum<'tokens> {
            #[allow(dropping_references)]
            fn reduce(slice: &[Self]) -> Option<(usize, Self)> {
                match &slice {
                    $(
                        $(
                            &[$($symbol_enum::$part($part_binding),)* ..] => {
                                Some((
                                    [$(drop($part_binding),)*].len(),
                                    $symbol_enum::$symbol(::std::rc::Rc::new($symbol::$symbol_case{$($part_binding: $part_binding.clone()),*}))
                                ))
                            },
                        )*
                    )*
                    _ => None
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, parser::Parser};

    crate::parser! {
        pub Token:
            LParen = r"[(]",
            RParen = r"[)]",
            Float = r"[+-]?[0-9][0-9_]*[.][0-9]+",
            Int = r"[+-]?[0-9][0-9_]*",
            OpAdd = r"[+]",
            OpSub = r"[-]";
        pub Symbol:
            Expr = Num(Num _num)
                 | Paren(LParen _lparen Expr _inner RParen _rparen)
                 | BinOp(Expr _lhs BinOp _op Expr _rhs),
            BinOp = Add(OpAdd _op)
                  | Sub(OpSub _op),
            Num = Int(Int _value)
                | Float(Float _value);
    }

    #[test]
    fn parse() {
        let lexer = Lexer::<Token>::new("(123 + 2_222_333.6 - 0.25)").unwrap();
        let mut parser = Parser::<Symbol>::new(lexer);
        println!("{:#?}", parser.parse());
    }
}
