use std::collections::VecDeque;

#[derive(Debug)]
pub struct Parser<SymbolType> {
    symbols: VecDeque<SymbolType>,
}

pub trait SymbolMetadata: Sized {
    fn reduce(slice: &[Self]) -> Option<(usize, Self)>;
}

impl<SymbolType: SymbolMetadata> Parser<SymbolType> {
    pub fn new(tokens: impl Iterator<Item = impl Into<SymbolType>>) -> Self {
        Self {
            symbols: tokens.map(Into::into).collect(),
        }
    }
    pub fn parse(&mut self) -> Result<SymbolType, String> {
        let mut stack = Vec::new();
        while self.symbols.len() > 1 {
            let symbol = SymbolMetadata::reduce(self.symbols.as_slices().0);
            if let Some((n, symbol)) = symbol {
                for _ in 0..n {
                    self.symbols.pop_front();
                }
                self.symbols.push_front(symbol);
                while let Some(symbol) = stack.pop() {
                    self.symbols.push_front(symbol);
                }
            } else if let Some(symbol) = self.symbols.pop_front() {
                stack.push(symbol);
            }
        }
        if let Some(symbol) = self.symbols.pop_front()
            && stack.is_empty()
        {
            Ok(symbol)
        } else {
            Err("Failed to parse input".to_string())
        }
    }
}

#[macro_export]
macro_rules! parser {
    (
        $vis:vis $symbol_enum:ident @ $token_type:ident{$($token_case:ident),*}:
            $($symbol:ident = $($symbol_case:ident($($part:ident $part_binding:ident)*))|*;)*
    ) => {
        $(
            #[derive(Debug, Clone)]
            $vis struct $token_case<'tokens>(#[allow(unused)] &'tokens str);
        )*
        $(
            #[derive(Debug, Clone)]
            $vis enum $symbol<'tokens> {
                $($symbol_case{$(#[allow(unused)] $part_binding: ::std::rc::Rc<$part<'tokens>>),*}),*,
            }
        )*
        #[derive(Debug, Clone)]
        $vis enum $symbol_enum<'tokens> {
            $(
                $token_case(::std::rc::Rc<$token_case<'tokens>>),
            )*
            $(
                $symbol(::std::rc::Rc<$symbol<'tokens>>),
            )*
        }
        impl <'tokens> From<$token_type<'tokens>> for $symbol_enum<'tokens> {
            fn from(token: $token_type<'tokens>) -> Self {
                match token {
                    $($token_type::$token_case(token) => $symbol_enum::$token_case(::std::rc::Rc::new($token_case(token))),)*
                    #[allow(unreachable_patterns)] _ => unimplemented!(),
                }
            }
        }
        impl <'tokens> $crate::parser::SymbolMetadata for $symbol_enum<'tokens> {
            fn reduce(slice: &[Self]) -> Option<(usize, Self)> {
                match &slice {
                    $(
                        $(
                            &[$($symbol_enum::$part($part_binding),)* ..] => {
                                Some((
                                    [$(#[allow(dropping_references)] drop($part_binding),)*].len(),
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

    crate::lexer! {
        pub Token:
            r"[(]" => LParen,
            r"[)]" => RParen,
            r"[+-]?[0-9][0-9_]*[.][0-9]+" => Float,
            r"[+-]?[0-9][0-9_]*" => Int,
            r"[+]" => OpAdd,
            r"[-]" => OpSub,
    }

    crate::parser! {
        pub Symbol @ Token{LParen, RParen, Int, Float, OpAdd, OpSub}:
            Expr = Num(Num num)
                 | Paren(LParen lparen Expr inner RParen rparen)
                 | BinOp(Expr lhs BinOp op Expr rhs);
            BinOp = OpAdd(OpAdd op)
                  | OpSub(OpSub op);
            Num = Int(Int value)
                | Float(Float value);
    }

    #[test]
    fn parse() {
        let lexer = Lexer::<Token>::new("(123 + 2_222_333.6 - 0.25)").unwrap();
        let mut parser = Parser::<Symbol>::new(lexer);
        println!("{:#?}", parser.parse());
    }
}
