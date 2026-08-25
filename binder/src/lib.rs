use flamelang::runner::Runner;
use flamelang::vm::Value;
use std::path::PathBuf;

pub struct Binder {
    pub runner: Runner,
}

impl Binder {
    pub fn load(path: &str) -> Result<Self, String> {
        let mut runner = Runner::new(PathBuf::from(path));

        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

        let mut lexer = flamelang::lexer::Lexer::new(&content);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            let is_eof = tok.kind == flamelang::lexer::TokenKind::EOF;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }

        let mut parser = flamelang::parser::Parser::new(tokens, path.to_string());
        let stmts = parser.parse().map_err(|e| e.message)?;

        runner.run(&stmts)?;

        Ok(Self { runner })
    }

    pub fn call(&mut self, func_name: &str, args: Vec<Value>) -> Result<Value, String> {
        let func_val = {
            let env = self.runner.env.lock().unwrap();
            env.get(func_name)
                .ok_or_else(|| format!("Function {} not found", func_name))?
        };
        self.runner.invoke_callback_value(&func_val, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_binder_load_and_call() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(
            b"
        export fn hello(name: String) -> String {
            return $\"Hello, {name}\";
        }
        "
        )
        .unwrap();

        let path = temp_file.path().to_str().unwrap();
        let mut binder = Binder::load(path).unwrap();

        let args = vec![Value::String("World".to_string())];
        let res = binder.call("hello", args).unwrap();

        match res {
            Value::String(s) => assert_eq!(s, "Hello, World"),
            _ => panic!("Expected String value, got {:?}", res),
        }
    }
}
