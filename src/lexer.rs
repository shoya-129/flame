#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Let,
    Const,
    Fn,
    Struct,
    Enum,
    Trait,
    Impl,
    Export,
    Import,
    In,
    Match,
    If,
    Else,
    For,
    While,
    Loop,
    Break,
    Continue,
    Defer,
    Async,
    Await,
    Thread,
    Yield,
    Return,
    Mut,
    As,
    SelfLower,
    SelfUpper,
    Type,
    Where,
    Formula,
    True,
    False,
    Nil,
    Comment,
    Newline,

    // Literals
    Identifier,
    IntLiteral,
    FloatLiteral,
    StringLiteral,

    // Interpolation Tokens
    InterpolatedStringStart,   // $"
    InterpolatedStringContent, // raw characters inside
    InterpolationStart,        // %{
    InterpolationEnd,          // }
    StringEnd,                 // "

    // Symbols & Operators
    Arrow,          // ->
    Equal,          // =
    EqualEqual,     // ==
    Plus,           // +
    PlusEqual,      // +=
    Minus,          // -
    MinusEqual,     // -=
    Star,           // *
    Slash,          // /
    Percent,        // %
    Ampersand,      // &
    Pipe,           // |
    Caret,          // ^
    Ampersand2,     // &&
    Pipe2,          // ||
    Dot,            // .
    Comma,          // ,
    Colon,          // :
    At,             // @
    Dollar,         // $
    Question,       // ?
    Exclamation,    // !
    OpenParen,      // (
    CloseParen,     // )
    OpenBracket,    // [
    CloseBracket,   // ]
    OpenBrace,      // {
    CloseBrace,     // }
    DoubleDot,      // ..
    DoubleDotEqual, // ..=
    Lt,
    Le,
    Gt,
    Ge,

    // End of File
    EOF,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    index: usize,
    line: usize,
    col: usize,
    // Stack to keep track of string interpolation state
    // true = inside an interpolation block `%{ ... }`, false = in a normal string
    pub interpolation_stack: Vec<bool>,
    pub keep_comments: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            col: 1,
            interpolation_stack: Vec::new(),
            keep_comments: false,
        }
    }

    fn peek(&self) -> Option<char> {
        if self.index < self.chars.len() {
            Some(self.chars[self.index])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.index + 1 < self.chars.len() {
            Some(self.chars[self.index + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.index < self.chars.len() {
            let ch = self.chars[self.index];
            self.index += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    pub fn next_token(&mut self) -> Token {
        let start_pos_before_skip = self.index;
        let start_line_before_skip = self.line;
        let start_col_before_skip = self.col;

        if let Some(kind) = self.skip_whitespace_and_comments() {
            return Token {
                kind,
                lexeme: self.source[start_pos_before_skip..self.index].to_string(),
                span: Span {
                    start: start_pos_before_skip,
                    end: self.index,
                    line: start_line_before_skip,
                    col: start_col_before_skip,
                },
            };
        }

        let start_pos = self.index;
        let start_line = self.line;
        let start_col = self.col;

        let ch = match self.advance() {
            Some(c) => c,
            None => {
                return Token {
                    kind: TokenKind::EOF,
                    lexeme: String::new(),
                    span: Span {
                        start: start_pos,
                        end: start_pos,
                        line: start_line,
                        col: start_col,
                    },
                };
            }
        };

        // If we are currently scanning inside an interpolated string, we have to look for %{ or "
        if let Some(false) = self.interpolation_stack.last() {
            // We are scanning the text part of a string
            if ch == '"' {
                self.interpolation_stack.pop();
                return Token {
                    kind: TokenKind::StringEnd,
                    lexeme: "\"".to_string(),
                    span: Span {
                        start: start_pos,
                        end: self.index,
                        line: start_line,
                        col: start_col,
                    },
                };
            } else if ch == '{' {
                // Toggle the top of the stack to true (we are inside an expression now)
                *self.interpolation_stack.last_mut().unwrap() = true;
                return Token {
                    kind: TokenKind::InterpolationStart,
                    lexeme: "{".to_string(),
                    span: Span {
                        start: start_pos,
                        end: self.index,
                        line: start_line,
                        col: start_col,
                    },
                };
            } else {
                // Read text until " or {
                let mut content = ch.to_string();
                while let Some(next) = self.peek() {
                    if next == '"' || next == '{' {
                        break;
                    }
                    content.push(self.advance().unwrap());
                }
                return Token {
                    kind: TokenKind::InterpolatedStringContent,
                    lexeme: content,
                    span: Span {
                        start: start_pos,
                        end: self.index,
                        line: start_line,
                        col: start_col,
                    },
                };
            }
        }

        // Standard character handling
        let kind = match ch {
            '(' => TokenKind::OpenParen,
            ')' => {
                // If we are in an interpolation expression block and hit a CloseBrace or CloseParen, we must handle it.
                TokenKind::CloseParen
            }
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            '{' => TokenKind::OpenBrace,
            '}' => {
                // Check if we are inside an interpolation expression block.
                // If the top of the stack is true, this '}' finishes the interpolation expression!
                if let Some(true) = self.interpolation_stack.last() {
                    *self.interpolation_stack.last_mut().unwrap() = false; // toggle back to scanning string text
                    TokenKind::InterpolationEnd
                } else {
                    TokenKind::CloseBrace
                }
            }
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '@' => TokenKind::At,
            '?' => TokenKind::Question,
            '!' => TokenKind::Exclamation,
            '+' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::PlusEqual
                } else {
                    TokenKind::Plus
                }
            },
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::MinusEqual
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => {
                if self.keep_comments && self.peek() == Some('/') {
                    self.advance();
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    TokenKind::Comment
                } else if self.keep_comments && self.peek() == Some('*') {
                    self.advance();
                    while let Some(ch) = self.peek() {
                        if ch == '*' && self.peek_next() == Some('/') {
                            self.advance();
                            self.advance();
                            break;
                        }
                        self.advance();
                    }
                    TokenKind::Comment
                } else {
                    TokenKind::Slash
                }
            }
            '%' => TokenKind::Percent,
            '^' => TokenKind::Caret,
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::DoubleDotEqual
                    } else {
                        TokenKind::DoubleDot
                    }
                } else {
                    TokenKind::Dot
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::Ampersand2
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::Pipe2
                } else {
                    TokenKind::Pipe
                }
            }
            '$' => {
                if self.peek() == Some('"') {
                    self.advance(); // consume '"'
                    self.interpolation_stack.push(false); // start scanning string text
                    TokenKind::InterpolatedStringStart
                } else {
                    TokenKind::Dollar
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '"' => {
                let mut content = String::new();

                while let Some(next) = self.peek() {
                    if next == '"' {
                        self.advance();
                        break;
                    }

                    content.push(self.advance().unwrap());
                }

                return Token {
                    kind: TokenKind::StringLiteral,
                    lexeme: content,
                    span: Span {
                        start: start_pos,
                        end: self.index,
                        line: start_line,
                        col: start_col,
                    },
                };
            }
            '\'' => {
                let mut content = String::new();

                while let Some(next) = self.peek() {
                    if next == '\'' {
                        self.advance();
                        break;
                    }

                    content.push(self.advance().unwrap());
                }

                return Token {
                    kind: TokenKind::StringLiteral,
                    lexeme: content,
                    span: Span {
                        start: start_pos,
                        end: self.index,
                        line: start_line,
                        col: start_col,
                    },
                };
            }
            _ => {
                if ch.is_ascii_digit() {
                    self.scan_number(ch)
                } else if ch.is_alphabetic() || ch == '_' {
                    self.scan_identifier(ch)
                } else {
                    // Unknown character, yield Identifier representing error or fallback
                    TokenKind::Identifier
                }
            }
        };

        let lexeme = self.source[start_pos..self.index].to_string();
        let mut resolved_kind = kind;
        if resolved_kind == TokenKind::Identifier {
            resolved_kind = Self::check_keyword(&lexeme);
        }

        Token {
            kind: resolved_kind,
            lexeme,
            span: Span {
                start: start_pos,
                end: self.index,
                line: start_line,
                col: start_col,
            },
        }
    }

    fn scan_number(&mut self, _first_char: char) -> TokenKind {
        let mut has_dot = false;
        while let Some(next) = self.peek() {
            if next.is_ascii_digit() {
                self.advance();
            } else if next == '.' && !has_dot {
                // Make sure it is not double dots `..` or `..=`
                if let Some(after) = self.peek_next() {
                    if after == '.' {
                        break;
                    }
                }
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }
        if has_dot {
            TokenKind::FloatLiteral
        } else {
            TokenKind::IntLiteral
        }
    }

    fn scan_identifier(&mut self, _first_char: char) -> TokenKind {
        while let Some(next) = self.peek() {
            if next.is_alphanumeric() || next == '_' {
                self.advance();
            } else {
                break;
            }
        }
        TokenKind::Identifier
    }

    pub fn check_keyword(lexeme: &str) -> TokenKind {
        match lexeme {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "fn" => TokenKind::Fn,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "trait" => TokenKind::Trait,
            "impl" => TokenKind::Impl,
            "export" => TokenKind::Export,
            "import" => TokenKind::Import,
            "in" => TokenKind::In,
            "match" => TokenKind::Match,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "loop" => TokenKind::Loop,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "defer" => TokenKind::Defer,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "thread" => TokenKind::Thread,
            "yield" => TokenKind::Yield,
            "return" => TokenKind::Return,
            "mut" => TokenKind::Mut,
            "as" => TokenKind::As,
            "self" => TokenKind::SelfLower,
            "Self" => TokenKind::SelfUpper,
            "type" => TokenKind::Type,
            "where" => TokenKind::Where,
            "formula" => TokenKind::Formula,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "nil" => TokenKind::Nil,
            _ => TokenKind::Identifier,
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Option<TokenKind> {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.advance();
                }
                Some('\n') | Some(';') => {
                    self.advance();
                    if self.keep_comments {
                        return Some(TokenKind::Newline);
                    }
                }
                Some('/') => {
                    if self.keep_comments {
                        break None;
                    }
                    if self.peek_next() == Some('/') {
                        // Line comment
                        self.advance();
                        self.advance();
                        while let Some(ch) = self.peek() {
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if self.peek_next() == Some('*') {
                        // Block comment
                        self.advance();
                        self.advance();
                        while let Some(ch) = self.peek() {
                            if ch == '*' && self.peek_next() == Some('/') {
                                self.advance();
                                self.advance();
                                break;
                            }
                            self.advance();
                        }
                    } else {
                        break None;
                    }
                }
                _ => break None,
            }
        }
    }
}
