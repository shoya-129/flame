use flamelang::lexer::Lexer;
use flamelang::parser::Parser;

fn main() {
    let src = "fn main() { x.abs().assert_eq(5) }";
    let mut lexer = Lexer::new(src);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = tok.kind == flamelang::lexer::TokenKind::EOF;
        tokens.push(tok);
        if is_eof { break; }
    }
    let mut parser = Parser::new(tokens, "test".to_string());
    match parser.parse() {
        Ok(stmts) => println!("{:#?}", stmts),
        Err(e) => println!("Error: {}", e.message),
    }
}
