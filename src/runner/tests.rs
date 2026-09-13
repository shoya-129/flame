use super::core::Runner;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::vm::*;
use std::path::PathBuf;

fn run_flame(code: &str) -> Result<Value, String> {
        let mut lexer = Lexer::new(code);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok.kind == crate::lexer::TokenKind::EOF {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }

        let mut parser = Parser::new(tokens, "test.flame".to_string());
        let stmts = parser.parse().map_err(|diag| diag.message)?;
        let mut runner = Runner::new(PathBuf::from("test.flame"));
        runner.run(&stmts)
    }

    #[test]
    fn mut_ref_through_enum_field() {
        let code = r#"fn change(&mut name: String) {
    print("in change, before:", name)
    name = "core"
    print("in change, after:", name)
}

enum Config {
    modules(Formula)
}

fn main() {
    let mut f: Formula = formula {
        name: "std",
        v: "1.0.0",
        description: "std lib of flame"
    }

    let mut con: Config = Config.modules(f)

    change(&mut con.name)
    print("final:", con.name)
}
main()"#;

        run_flame(code).unwrap();
    }

    #[test]
    fn std_thread_execution() {
        let code = r#"import std.thread

fn main() {
    let (tx, rx) = thread.channel()
    tx.send("test_message")
    rx.recv()
}
main()"#;

        let result = run_flame(code).unwrap();
        assert_eq!(result.to_string(), "test_message");
    }

    #[test]
    fn annotation_decl_and_stripping_test() {
        let code = r#"
annotation Benchmark(name: String) -> Formula {
    return formula { name: name }
}

@Test
fn test_my_func() {
    return 42
}

fn main() -> i64 {
    return 100
}
main()
"#;
        let result = run_flame(code).unwrap();
        assert_eq!(result.to_string(), "100");
    }

    #[test]
    fn let_decl_annotation_executes() {
        let code = r#"
annotation Entity(table: String) -> String {
    print("Registering entity")
    return table
}

@Entity(table: "users")
let User = formula {
    id: 9
    name: "9"
}
"#;
        let mut lexer = Lexer::new(code);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok.kind == crate::lexer::TokenKind::EOF {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }

        let mut parser = Parser::new(tokens, "test.flame".to_string());
        let stmts = parser.parse().map_err(|diag| diag.message).unwrap();
        let mut runner = Runner::new(PathBuf::from("test.flame"));
        let result = runner.run(&stmts).unwrap();
        assert_eq!(result.to_string(), "nil");
    }

    #[test]
    fn explicit_type_conversion_methods_test() {
        let code = r#"
fn main() {
    let num_str = "42"
    let val = num_str.toInt()
    let hex = "1A".toInt(16)
    let flt = "3.14159".toFloat()
    let prec = 3.14159.toString(2)
    let bool_val = "true".toBool()
    return val + hex
}
main()
"#;
        let result = run_flame(code).unwrap();
        assert_eq!(result.to_string(), "68"); // 42 + 26 = 68
    }

    #[test]
    fn custom_annotation_logger_test() {
        let code = r#"
export annotation Logger(prefix: String) -> String {
    print($"[LOGGER INIT] Prefix configured: {prefix}")
    prefix
}

@Logger(prefix: "flame-cli")
fn main() {
    print("Inside main")
}
"#;
        let mut lexer = Lexer::new(code);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok.kind == crate::lexer::TokenKind::EOF {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }
        let mut parser = Parser::new(tokens, "test.flame".to_string());
        let stmts = parser.parse().map_err(|diag| diag.message).unwrap();
        let mut runner = Runner::new(PathBuf::from("test.flame"));
        let result = runner.run(&stmts);
        assert!(result.is_ok());
    }

    