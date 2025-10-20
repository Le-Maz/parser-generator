use parser_generator::{grammar, lexer::Lexer, parser::Parser};

grammar! {
    Token:
        LParen = r"[(]",
        RParen = r"[)]",
        Float = r"[-+]?[0-9]+[.][0-9]+",
        Int = r"[-+]?[0-9]+",
        Str = r#"".*?[^\\]"|"""#;
    Symbol:
        Expr = List(List _list) | Literal(Literal _literal),
        Literal = String(Str _string) | Num(Num _num),
        Num = Float(Float _value) | Int(Int _value),
        List = List(ListContent _content RParen _rparen),
        ListContent = Node(ListContent _content Expr _next_expr) | Leaf(LParen _lparen);
}

#[test]
fn parse() {
    let lexer = Lexer::<Token>::new(include_str!("example_s_expression.txt")).unwrap();
    let mut parser = Parser::<Symbol>::new(lexer);
    let _symbol = parser.parse().unwrap();
}
