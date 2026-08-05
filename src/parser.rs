#![allow(dead_code)]
use crate::diagnostics::Diagnostic;
use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Assign,
    PlusAssign,
    MinusAssign,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Range,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_name: String,
    pub default_val: Option<Expr>,
    pub is_ref: bool,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: String,
    pub pattern_span: Span,
    pub destructure: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralValue, Span),
    Identifier(String, Span),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Span),
    Call(Box<Expr>, Vec<(Option<String>, Expr)>, Span),
    Dot(Box<Expr>, String, Span),
    Formula(Vec<(String, Expr)>, Span),
    ThreadSpawn(Box<Expr>, Span),
    Closure {
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Stmt>,
        span: Span,
    },
    Await(Box<Expr>, Span),
    Tuple(Vec<Expr>, Span),
    VectorLiteral(Vec<Expr>, Span),
    InterpolatedString(Vec<InterpolatedSegment>, Span),
    Block(Vec<Stmt>, Span),
    Borrow(Box<Expr>, bool, Span),
    StructInit(Box<Expr>, Vec<(String, Expr)>, Span),
}

#[derive(Debug, Clone)]
pub enum InterpolatedSegment {
    Text(String),
    Expr(Expr),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, s) => s.clone(),
            Expr::Identifier(_, s) => s.clone(),
            Expr::Binary(_, _, _, s) => s.clone(),
            Expr::Call(_, _, s) => s.clone(),
            Expr::Dot(_, _, s) => s.clone(),
            Expr::Formula(_, s) => s.clone(),
            Expr::ThreadSpawn(_, s) => s.clone(),
            Expr::Closure { span, .. } => span.clone(),
            Expr::Await(_, s) => s.clone(),
            Expr::Tuple(_, s) => s.clone(),
            Expr::VectorLiteral(_, s) => s.clone(),
            Expr::InterpolatedString(_, s) => s.clone(),
            Expr::Block(_, s) => s.clone(),
            Expr::Borrow(_, _, s) => s.clone(),
            Expr::StructInit(_, _, s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Unit(String),
    Tuple(String, Vec<String>),
    Struct(String, Vec<(String, String)>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    LetDecl {
        name: String,
        is_mut: bool,
        type_ann: Option<String>,
        value: Expr,
        annotations: Vec<Annotation>,
        span: Span,
    },
    ConstDecl {
        name: String,
        is_mut: bool,
        type_ann: Option<String>,
        value: Expr,
        annotations: Vec<Annotation>,
        span: Span,
    },
    FuncDecl {
        name: String,
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Stmt>,
        annotations: Vec<Annotation>,
        span: Span,
    },
    AnnotationDecl {
        name: String,
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Stmt>,
        span: Span,
    },
    StructDecl {
        name: String,
        fields: Vec<(String, String)>,
        annotations: Vec<Annotation>,
        span: Span,
    },
    EnumDecl {
        name: String,
        variants: Vec<EnumVariant>,
        annotations: Vec<Annotation>,
        span: Span,
    },
    TraitDecl {
        name: String,
        signatures: Vec<String>,
        span: Span,
    },
    ImplDecl {
        trait_name: Option<String>,
        target_type: String,
        methods: Vec<Stmt>,
        span: Span,
    },
    ImportDecl {
        path: Vec<String>,
        glob: bool,
        span: Span,
    },
    ExportDecl(Box<Stmt>, Span),
    ExprStmt(Expr),
    IfStmt {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    ForStmt {
        var_name: String,
        iterable: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    WhileStmt {
        cond: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    LoopStmt {
        body: Vec<Stmt>,
        span: Span,
    },
    MatchStmt {
        target: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },
    ReturnStmt(Option<Expr>, Span),
    DeferStmt(Box<Stmt>, Span),
    Break(Span),
    Continue(Span),
    PluginDecl {
        name: String,
        span: Span,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
    filepath: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, filepath: String) -> Self {
        Self {
            tokens,
            index: 0,
            filepath,
        }
    }

    fn peek(&self) -> Token {
        if self.index < self.tokens.len() {
            self.tokens[self.index].clone()
        } else {
            self.tokens[self.tokens.len() - 1].clone()
        }
    }

    fn advance(&mut self) -> Token {
        let current = self.peek();
        if self.index < self.tokens.len() {
            self.index += 1;
        }
        current
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn check_next(&self, kind: TokenKind) -> bool {
        if self.index + 1 < self.tokens.len() {
            self.tokens[self.index + 1].kind == kind
        } else {
            false
        }
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<Token, Diagnostic> {
        let token = self.peek();
        if token.kind == kind {
            Ok(self.advance())
        } else {
            Err(Diagnostic::new_error(
                format!("expected {:?}, found '{}'", kind, token.lexeme),
                self.filepath.clone(),
                token.span.clone(),
                Some(msg.to_string()),
                Some(format!(
                    "Insert '{}' here to fix syntax error",
                    token.lexeme
                )),
            ))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        let mut statements = Vec::new();
        while !self.check(TokenKind::EOF) {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, Diagnostic> {
        // Check for standalone @plugin directive
        if self.check(TokenKind::At) && self.check_next(TokenKind::Identifier) {
            let next_tok = &self.tokens[self.index + 1];
            if next_tok.lexeme == "plugin" {
                let at_tok = self.consume(TokenKind::At, "")?;
                self.consume(TokenKind::Identifier, "")?; // consume "plugin"
                let name_tok = if self.check(TokenKind::StringLiteral) {
                    self.consume(TokenKind::StringLiteral, "")?
                } else {
                    self.consume(TokenKind::Identifier, "expected plugin name or path")?
                };
                let span = Span {
                    start: at_tok.span.start,
                    end: name_tok.span.end,
                    line: at_tok.span.line,
                    col: at_tok.span.col,
                };
                return Ok(Stmt::PluginDecl {
                    name: name_tok.lexeme.trim_matches('"').to_string(),
                    span,
                });
            }
        }

        // Handle annotations
        let mut annotations = Vec::new();
        while self.check(TokenKind::At) {
            annotations.push(self.parse_annotation()?);
        }

        let token = self.peek();
        match token.kind {
            TokenKind::Async => {
                self.advance(); // consume "async"
                self.parse_func_decl(annotations)
            }
            TokenKind::Import => self.parse_import_statement(),
            TokenKind::Export => self.parse_export_statement(annotations),
            TokenKind::Let => self.parse_var_decl(TokenKind::Let, annotations),
            TokenKind::Const => self.parse_var_decl(TokenKind::Const, annotations),
            TokenKind::Fn => self.parse_func_decl(annotations),
            TokenKind::Annotation => self.parse_annotation_decl(),
            TokenKind::Struct => self.parse_struct_decl(annotations),
            TokenKind::Enum => self.parse_enum_decl(annotations),
            TokenKind::Impl => self.parse_impl_decl(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Loop => self.parse_loop_statement(),
            TokenKind::Match => self.parse_match_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Defer => self.parse_defer_statement(),
            TokenKind::Break => {
                let tok = self.advance();
                Ok(Stmt::Break(tok.span.clone()))
            }
            TokenKind::Continue => {
                let tok = self.advance();
                Ok(Stmt::Continue(tok.span.clone()))
            }
            _ => {
                // Expression statement
                let expr = self.parse_expr()?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    fn parse_annotation(&mut self) -> Result<Annotation, Diagnostic> {
        self.consume(TokenKind::At, "expected '@' decorator prefix")?;
        let id = self.consume(TokenKind::Identifier, "expected annotation name")?;
        let mut name = id.lexeme.clone();
        while self.match_token(TokenKind::Dot) {
            let next_id = self.consume(TokenKind::Identifier, "expected property name after '.'")?;
            name.push('.');
            name.push_str(&next_id.lexeme);
        }
        let mut args = Vec::new();

        if self.match_token(TokenKind::OpenParen) {
            while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::EOF) {
                let mut arg_str = String::new();
                let mut depth = 0;
                while !self.check(TokenKind::EOF) {
                    if depth == 0
                        && (self.check(TokenKind::Comma) || self.check(TokenKind::CloseParen))
                    {
                        break;
                    }
                    let tok = self.peek();
                    match tok.kind {
                        TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace => {
                            depth += 1
                        }
                        TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace => {
                            if depth > 0 {
                                depth -= 1;
                            } else {
                                break;
                            }
                        }
                        _ => {}
                    }
                    if !arg_str.is_empty()
                        && tok.kind != TokenKind::Comma
                        && !arg_str.ends_with(' ')
                        && !arg_str.ends_with('(')
                        && !arg_str.ends_with('[')
                        && !arg_str.ends_with(':')
                    {
                        arg_str.push(' ');
                    }
                    if tok.kind == TokenKind::StringLiteral {
                        arg_str.push('"');
                        arg_str.push_str(&tok.lexeme);
                        arg_str.push('"');
                    } else {
                        arg_str.push_str(&tok.lexeme);
                    }
                    self.advance();
                }
                let trimmed = arg_str.trim().to_string();
                if !trimmed.is_empty() {
                    args.push(trimmed);
                }
                self.match_token(TokenKind::Comma);
            }
            self.consume(
                TokenKind::CloseParen,
                "expected ')' to close annotation arguments",
            )?;
        }

        Ok(Annotation { name, args })
    }

    fn parse_import_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Import, "expected 'import' keyword")?;
        let mut path = Vec::new();
        let mut glob = false;

        let first = self.advance(); // consume any token (allows keywords like thread/process)
        path.push(first.lexeme.clone());

        while self.match_token(TokenKind::Dot) {
            if self.match_token(TokenKind::Star) {
                glob = true;
                break;
            }
            let next = self.advance(); // consume any token
            path.push(next.lexeme.clone());
        }

        let end_span = self.peek().span.clone();
        Ok(Stmt::ImportDecl {
            path,
            glob,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_export_statement(&mut self, outer_annotations: Vec<Annotation>) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Export, "expected 'export' keyword")?;
        let mut inner = self.parse_statement()?;
        if !outer_annotations.is_empty() {
            match &mut inner {
                Stmt::FuncDecl { annotations, .. }
                | Stmt::LetDecl { annotations, .. }
                | Stmt::ConstDecl { annotations, .. }
                | Stmt::StructDecl { annotations, .. }
                | Stmt::EnumDecl { annotations, .. } => {
                    let mut combined = outer_annotations;
                    combined.append(annotations);
                    *annotations = combined;
                }
                _ => {}
            }
        }
        let end_span = inner.span();
        Ok(Stmt::ExportDecl(
            Box::new(inner),
            Span {
                start: start_tok.span.start,
                end: end_span.end,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        ))
    }

    fn parse_type(&mut self) -> Result<String, Diagnostic> {
        let mut t = String::new();
        if self.match_token(TokenKind::OpenParen) {
            t.push('(');
            let mut sub_types = Vec::new();
            while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::EOF) {
                sub_types.push(self.parse_type()?);
                self.match_token(TokenKind::Comma);
            }
            self.consume(TokenKind::CloseParen, "expected ')' to close tuple type")?;
            t.push_str(&sub_types.join(", "));
            t.push(')');
        } else if self.match_token(TokenKind::OpenBracket) {
            t.push('[');
            let inner = self.parse_type()?;
            self.consume(TokenKind::CloseBracket, "expected ']' to close list type")?;
            t.push_str(&inner);
            t.push(']');
        } else {
            let tok = self.advance();
            t.push_str(&tok.lexeme);
        }

        if self.match_token(TokenKind::Lt) {
            t.push('<');
            let mut sub_types = Vec::new();
            while !self.check(TokenKind::Gt) && !self.check(TokenKind::EOF) {
                sub_types.push(self.parse_type()?);
                self.match_token(TokenKind::Comma);
            }
            self.consume(
                TokenKind::Gt,
                "expected '>' to close generic type arguments",
            )?;
            t.push_str(&sub_types.join(", "));
            t.push('>');
        }

        if self.match_token(TokenKind::Arrow) {
            t.push_str(" -> ");
            t.push_str(&self.parse_type()?);
        }
        Ok(t)
    }

    fn parse_var_decl(
        &mut self,
        kind: TokenKind,
        annotations: Vec<Annotation>,
    ) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(kind.clone(), "expected variable keyword")?;

        let mut is_mut = false;
        let name = if self.match_token(TokenKind::OpenParen) {
            let mut items = Vec::new();
            while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::EOF) {
                let id = self.consume(
                    TokenKind::Identifier,
                    "expected variable identifier in destructuring",
                )?;
                items.push(id.lexeme.clone());
                if self.match_token(TokenKind::Comma) {
                    if self.check(TokenKind::CloseParen) {
                        return Err(Diagnostic::new_error(
                            "trailing comma in destructuring without identifier".to_string(),
                            self.filepath.clone(),
                            id.span.clone(),
                            Some("expected variable name after ','".to_string()),
                            Some("Remove the comma or add another variable name".to_string()),
                        ));
                    }
                }
            }
            self.consume(TokenKind::CloseParen, "expected ')' to close destructuring")?;
            format!("({})", items.join(", "))
        } else {
            if self.match_token(TokenKind::Mut) {
                is_mut = true;
            }
            let name_tok = self.consume(TokenKind::Identifier, "expected variable name")?;
            name_tok.lexeme.clone()
        };

        let mut type_ann = None;
        if self.match_token(TokenKind::Colon) {
            let t = self.parse_type()?;
            type_ann = Some(t);
        }

        self.consume(TokenKind::Equal, "expected '=' before value assignment")?;
        let value = self.parse_expr()?;
        let end_span = value.span();

        let decl_span = Span {
            start: start_tok.span.start,
            end: end_span.end,
            line: start_tok.span.line,
            col: start_tok.span.col,
        };

        match kind {
            TokenKind::Let => Ok(Stmt::LetDecl {
                name,
                is_mut,
                type_ann,
                value,
                annotations,
                span: decl_span,
            }),
            TokenKind::Const => Ok(Stmt::ConstDecl {
                name,
                is_mut: false,
                type_ann,
                value,
                annotations,
                span: decl_span,
            }),
            _ => unreachable!(),
        }
    }

    fn parse_func_decl(&mut self, annotations: Vec<Annotation>) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Fn, "expected 'fn' function definition")?;
        let name_tok = self.consume(TokenKind::Identifier, "expected function name")?;
        let name = name_tok.lexeme.clone();

        if self.match_token(TokenKind::Lt) {
            while !self.check(TokenKind::Gt) && !self.check(TokenKind::EOF) {
                self.advance();
            }
            self.consume(TokenKind::Gt, "expected '>' after generic type parameters")?;
        }

        self.consume(TokenKind::OpenParen, "expected '(' for parameters list")?;
        let mut params = Vec::new();
        while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::EOF) {
            let mut is_ref = false;
            let mut is_mut = false;

            if self.match_token(TokenKind::Ampersand) {
                is_ref = true;

                if self.match_token(TokenKind::Mut) {
                    is_mut = true;
                }
            }

            let p_name_tok = if self.check(TokenKind::SelfLower) {
                self.advance()
            } else {
                self.consume(TokenKind::Identifier, "expected parameter name")?
            };

            let p_type;
            if self.match_token(TokenKind::Colon) {
                p_type = self.parse_type()?;
            } else if p_name_tok.kind != TokenKind::SelfLower {
                return Err(Diagnostic::new_error(
                    "expected ':' after parameter name".to_string(),
                    self.filepath.clone(),
                    p_name_tok.span.clone(),
                    Some("Add a type annotation for this parameter".to_string()),
                    Some("Use ': Type' after the parameter name".to_string()),
                ));
            } else {
                p_type = "Self".to_string();
            }

            let mut default_val = None;
            if self.match_token(TokenKind::Equal) {
                default_val = Some(self.parse_expr()?);
            }

            params.push(Param {
                name: p_name_tok.lexeme.clone(),
                type_name: p_type,
                default_val,
                is_ref,
                is_mut,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        self.consume(
            TokenKind::CloseParen,
            "expected ')' to close parameters list",
        )?;

        let mut return_type = None;
        if self.match_token(TokenKind::Arrow) {
            let ret = self.parse_type()?;
            return_type = Some(ret);
        }

        let body = self.parse_block()?;
        let end_span = self.peek().span.clone();

        Ok(Stmt::FuncDecl {
            name,
            params,
            return_type,
            body,
            annotations,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_annotation_decl(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Annotation, "expected 'annotation' keyword")?;
        let name_tok = self.consume(TokenKind::Identifier, "expected annotation function name")?;
        let name = name_tok.lexeme.clone();

        if self.match_token(TokenKind::Lt) {
            while !self.check(TokenKind::Gt) && !self.check(TokenKind::EOF) {
                self.advance();
            }
            self.consume(TokenKind::Gt, "expected '>' after generic type parameters")?;
        }

        self.consume(TokenKind::OpenParen, "expected '(' for parameters list")?;
        let mut params = Vec::new();
        while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::EOF) {
            let mut is_ref = false;
            let mut is_mut = false;

            if self.match_token(TokenKind::Ampersand) {
                is_ref = true;
                if self.match_token(TokenKind::Mut) {
                    is_mut = true;
                }
            }

            let p_name_tok = self.consume(TokenKind::Identifier, "expected parameter name")?;
            let p_type;
            if self.match_token(TokenKind::Colon) {
                p_type = self.parse_type()?;
            } else {
                return Err(Diagnostic::new_error(
                    "expected ':' after parameter name".to_string(),
                    self.filepath.clone(),
                    p_name_tok.span.clone(),
                    Some("Add a type annotation for this parameter".to_string()),
                    Some("Use ': Type' after the parameter name".to_string()),
                ));
            }

            let mut default_val = None;
            if self.match_token(TokenKind::Equal) {
                default_val = Some(self.parse_expr()?);
            }

            params.push(Param {
                name: p_name_tok.lexeme.clone(),
                type_name: p_type,
                default_val,
                is_ref,
                is_mut,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        self.consume(
            TokenKind::CloseParen,
            "expected ')' to close parameters list",
        )?;

        let mut return_type = None;
        if self.match_token(TokenKind::Arrow) {
            let ret = self.parse_type()?;
            return_type = Some(ret);
        }

        let body = self.parse_block()?;
        let end_span = self.peek().span.clone();

        Ok(Stmt::AnnotationDecl {
            name,
            params,
            return_type,
            body,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_struct_decl(&mut self, annotations: Vec<Annotation>) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Struct, "expected 'struct'")?;
        let name_tok = self.consume(TokenKind::Identifier, "expected struct name")?;
        let name = name_tok.lexeme.clone();

        self.consume(TokenKind::OpenBrace, "expected '{'")?;
        let mut fields = Vec::new();
        while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
            let field_name = self.consume(TokenKind::Identifier, "expected field name")?;
            self.consume(TokenKind::Colon, "expected ':'")?;
            let field_type = self.consume(TokenKind::Identifier, "expected field type")?;
            fields.push((field_name.lexeme.clone(), field_type.lexeme.clone()));
            self.match_token(TokenKind::Comma);
        }
        self.consume(TokenKind::CloseBrace, "expected '}'")?;
        let end_span = self.peek().span.clone();

        Ok(Stmt::StructDecl {
            name,
            fields,
            annotations,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_enum_decl(&mut self, annotations: Vec<Annotation>) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Enum, "expected 'enum'")?;
        let name_tok = self.consume(TokenKind::Identifier, "expected enum name")?;
        let name = name_tok.lexeme.clone();

        self.consume(TokenKind::OpenBrace, "expected '{'")?;
        let mut variants = Vec::new();
        while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
            let var_tok = self.consume(TokenKind::Identifier, "expected variant name")?;
            let var_name = var_tok.lexeme.clone();

            if self.match_token(TokenKind::OpenParen) {
                let mut tuple_types = Vec::new();
                while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::EOF) {
                    tuple_types.push(self.parse_type()?);
                    self.match_token(TokenKind::Comma);
                }
                self.consume(TokenKind::CloseParen, "expected ')'")?;
                variants.push(EnumVariant::Tuple(var_name, tuple_types));
            } else if self.match_token(TokenKind::OpenBrace) {
                let mut struct_fields = Vec::new();
                while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
                    let field_tok = self.consume(
                        TokenKind::Identifier,
                        "expected field name in struct variant",
                    )?;
                    self.consume(TokenKind::Colon, "expected ':' after field name")?;
                    let field_type = self.parse_type()?;
                    struct_fields.push((field_tok.lexeme.clone(), field_type));
                    self.match_token(TokenKind::Comma);
                }
                self.consume(TokenKind::CloseBrace, "expected '}'")?;
                variants.push(EnumVariant::Struct(var_name, struct_fields));
            } else {
                variants.push(EnumVariant::Unit(var_name));
            }

            self.match_token(TokenKind::Comma);
        }
        self.consume(TokenKind::CloseBrace, "expected '}'")?;
        let end_span = self.peek().span.clone();

        Ok(Stmt::EnumDecl {
            name,
            variants,
            annotations,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_impl_decl(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Impl, "expected 'impl'")?;
        let name_tok =
            self.consume(TokenKind::Identifier, "expected implementation target type")?;
        let mut trait_name = None;
        let mut target_type = name_tok.lexeme.clone();

        if self.match_token(TokenKind::For) {
            trait_name = Some(target_type);
            let target_tok = self.consume(TokenKind::Identifier, "expected target struct name")?;
            target_type = target_tok.lexeme.clone();
        }

        self.consume(TokenKind::OpenBrace, "expected '{'")?;
        let mut methods = Vec::new();
        while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
            methods.push(self.parse_statement()?);
        }
        self.consume(TokenKind::CloseBrace, "expected '}'")?;
        let end_span = self.peek().span.clone();

        Ok(Stmt::ImplDecl {
            trait_name,
            target_type,
            methods,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::If, "expected 'if'")?;
        let cond = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let mut else_branch = None;

        if self.match_token(TokenKind::Else) {
            if self.check(TokenKind::If) {
                else_branch = Some(vec![self.parse_if_statement()?]);
            } else {
                else_branch = Some(self.parse_block()?);
            }
        }

        let end_span = self.peek().span.clone();
        Ok(Stmt::IfStmt {
            cond,
            then_branch,
            else_branch,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::For, "expected 'for'")?;
        let var_tok = self.consume(TokenKind::Identifier, "expected loop variable identifier")?;
        let var_name = var_tok.lexeme.clone();
        self.consume(TokenKind::In, "expected 'in' loop generator")?;
        let iterable = self.parse_expr()?;
        let body = self.parse_block()?;
        let end_span = self.peek().span.clone();

        Ok(Stmt::ForStmt {
            var_name,
            iterable,
            body,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::While, "expected 'while'")?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        let end_span = self.peek().span.clone();

        Ok(Stmt::WhileStmt {
            cond,
            body,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_loop_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Loop, "expected 'loop'")?;
        let body = self.parse_block()?;
        let end_span = self.peek().span.clone();

        Ok(Stmt::LoopStmt {
            body,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_match_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Match, "expected 'match'")?;
        let target = self.parse_expr()?;
        self.consume(TokenKind::OpenBrace, "expected '{'")?;
        let mut arms = Vec::new();
        while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
            let pat_tok = self.peek();
            let pat = pat_tok.lexeme.clone();
            let pattern_span = pat_tok.span.clone();
            self.advance();
            let mut destructure = Vec::new();
            if self.match_token(TokenKind::OpenBrace) {
                while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
                    let field = self.consume(TokenKind::Identifier, "expected identifier in pattern destructuring")?;
                    destructure.push(field.lexeme.clone());
                    self.match_token(TokenKind::Comma);
                }
                self.consume(TokenKind::CloseBrace, "expected '}' closing pattern destructuring")?;
            }
            if !self.match_token(TokenKind::FatArrow) && !self.match_token(TokenKind::Arrow) {
                return Err(self.consume(TokenKind::FatArrow, "expected '=>' pattern arm arrow").unwrap_err());
            }
            let body = self.parse_expr()?;
            arms.push(MatchArm {
                pattern: pat,
                pattern_span,
                destructure,
                body,
            });
            self.match_token(TokenKind::Comma);
        }
        self.consume(TokenKind::CloseBrace, "expected '}'")?;
        let end_span = self.peek().span.clone();

        Ok(Stmt::MatchStmt {
            target,
            arms,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Return, "expected 'return'")?;
        let mut val = None;
        if !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
            val = Some(self.parse_expr()?);
        }
        let end_span = self.peek().span.clone();

        Ok(Stmt::ReturnStmt(
            val,
            Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        ))
    }

    fn parse_defer_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let start_tok = self.consume(TokenKind::Defer, "expected 'defer'")?;
        let inner = self.parse_statement()?;
        let end_span = inner.span();
        Ok(Stmt::DeferStmt(
            Box::new(inner),
            Span {
                start: start_tok.span.start,
                end: end_span.end,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        ))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        self.consume(TokenKind::OpenBrace, "expected '{' for code block")?;
        let mut statements = Vec::new();
        while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
            statements.push(self.parse_statement()?);
        }
        self.consume(TokenKind::CloseBrace, "expected '}' to close code block")?;
        Ok(statements)
    }

    pub fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_or()?;
        let op = if self.match_token(TokenKind::Equal) {
            Some(BinaryOp::Assign)
        } else if self.match_token(TokenKind::PlusEqual) {
            Some(BinaryOp::PlusAssign)
        } else if self.match_token(TokenKind::MinusEqual) {
            Some(BinaryOp::MinusAssign)
        } else {
            None
        };

        if let Some(binary_op) = op {
            let value = self.parse_assignment()?;
            let span = Span {
                start: expr.span().start,
                end: value.span().end,
                line: expr.span().line,
                col: expr.span().col,
            };
            expr = Expr::Binary(Box::new(expr), binary_op, Box::new(value), span);
        }
        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_and()?;
        while self.match_token(TokenKind::Pipe2) {
            let right = self.parse_and()?;
            let span = Span {
                start: expr.span().start,
                end: right.span().end,
                line: expr.span().line,
                col: expr.span().col,
            };
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(right), span);
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_equality()?;
        while self.match_token(TokenKind::Ampersand2) {
            let right = self.parse_equality()?;
            let span = Span {
                start: expr.span().start,
                end: right.span().end,
                line: expr.span().line,
                col: expr.span().col,
            };
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right), span);
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = if self.match_token(TokenKind::EqualEqual) {
                Some(BinaryOp::Eq)
            } else if self.match_token(TokenKind::ExclamationEqual) {
                Some(BinaryOp::Ne)
            } else {
                None
            };
            if let Some(binary_op) = op {
                let right = self.parse_comparison()?;
                let span = Span {
                    start: expr.span().start,
                    end: right.span().end,
                    line: expr.span().line,
                    col: expr.span().col,
                };
                expr = Expr::Binary(Box::new(expr), binary_op, Box::new(right), span);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_range()?;
        while self.check(TokenKind::Lt)
            || self.check(TokenKind::Le)
            || self.check(TokenKind::Gt)
            || self.check(TokenKind::Ge)
        {
            // Mapped comparison operators
            let tok = self.advance();
            let op = match tok.kind {
                TokenKind::Le => BinaryOp::Le,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::Ge => BinaryOp::Ge,
                _ => BinaryOp::Lt,
            };
            let right = self.parse_range()?;
            let span = Span {
                start: expr.span().start,
                end: right.span().end,
                line: expr.span().line,
                col: expr.span().col,
            };
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }
        Ok(expr)
    }

    fn parse_range(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_term()?;
        if self.match_token(TokenKind::DoubleDot) {
            let right = self.parse_term()?;
            let span = Span {
                start: expr.span().start,
                end: right.span().end,
                line: expr.span().line,
                col: expr.span().col,
            };
            expr = Expr::Binary(Box::new(expr), BinaryOp::Range, Box::new(right), span);
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_factor()?;
        while self.check(TokenKind::Plus) || self.check(TokenKind::Minus) {
            let tok = self.advance();
            let op = match tok.kind {
                TokenKind::Plus => BinaryOp::Add,
                _ => BinaryOp::Sub,
            };
            let right = self.parse_factor()?;
            let span = Span {
                start: expr.span().start,
                end: right.span().end,
                line: expr.span().line,
                col: expr.span().col,
            };
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_unary()?;
        while self.check(TokenKind::Star) || self.check(TokenKind::Slash) {
            let tok = self.advance();
            let op = match tok.kind {
                TokenKind::Star => BinaryOp::Mul,
                _ => BinaryOp::Div,
            };
            let right = self.parse_unary()?;
            let span = Span {
                start: expr.span().start,
                end: right.span().end,
                line: expr.span().line,
                col: expr.span().col,
            };
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, Diagnostic> {
        if self.check(TokenKind::Ampersand) {
            let tok = self.advance();
            let mut is_mut = false;
            if self.match_token(TokenKind::Mut) {
                is_mut = true;
            }

            let expr = self.parse_unary()?;
            let span = Span {
                start: tok.span.start,
                end: expr.span().end,
                line: tok.span.line,
                col: tok.span.col,
            };
            return Ok(Expr::Borrow(Box::new(expr), is_mut, span));
        }
        if self.check(TokenKind::Minus) || self.check(TokenKind::Exclamation) {
            let tok = self.advance();
            let expr = self.parse_unary()?;
            let span = Span {
                start: tok.span.start,
                end: expr.span().end,
                line: tok.span.line,
                col: tok.span.col,
            };
            return Ok(Expr::Binary(
                Box::new(expr),
                BinaryOp::Sub,
                Box::new(Expr::Literal(LiteralValue::Int(0), span.clone())),
                span,
            ));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.peek();
        match token.kind {
            TokenKind::IntLiteral => {
                let tok = self.advance();
                let val = tok.lexeme.parse::<i64>().unwrap_or(0);
                let expr = Expr::Literal(LiteralValue::Int(val), tok.span.clone());
                self.parse_accessors(expr)
            }
            TokenKind::FloatLiteral => {
                let tok = self.advance();
                let val = tok.lexeme.parse::<f64>().unwrap_or(0.0);
                let expr = Expr::Literal(LiteralValue::Float(val), tok.span.clone());
                self.parse_accessors(expr)
            }
            TokenKind::StringLiteral => {
                let tok = self.advance();
                let expr =
                    Expr::Literal(LiteralValue::String(tok.lexeme.clone()), tok.span.clone());
                self.parse_accessors(expr)
            }
            TokenKind::True => {
                let tok = self.advance();
                let expr = Expr::Literal(LiteralValue::Bool(true), tok.span.clone());
                self.parse_accessors(expr)
            }
            TokenKind::False => {
                let tok = self.advance();
                let expr = Expr::Literal(LiteralValue::Bool(false), tok.span.clone());
                self.parse_accessors(expr)
            }
            TokenKind::Nil => {
                let tok = self.advance();
                let expr = Expr::Literal(LiteralValue::Nil, tok.span.clone());
                self.parse_accessors(expr)
            }
            TokenKind::Identifier | TokenKind::SelfLower => {
                let tok = self.advance();
                let expr = Expr::Identifier(tok.lexeme.clone(), tok.span.clone());
                self.parse_accessors(expr)
            }
            TokenKind::Thread => {
                if self.check_next(TokenKind::Dot) {
                    let tok = self.advance();
                    let expr = Expr::Identifier(tok.lexeme.clone(), tok.span.clone());
                    self.parse_accessors(expr)
                } else {
                    let start_tok = self.advance();
                    let block_expr = if self.check(TokenKind::OpenBrace) {
                        let start_brace = self.peek().span.clone();
                        let block_stmts = self.parse_block()?;
                        let end_brace = self.peek().span.clone();
                        Expr::Block(
                            block_stmts,
                            Span {
                                start: start_brace.start,
                                end: end_brace.start,
                                line: start_brace.line,
                                col: start_brace.col,
                            },
                        )
                    } else {
                        self.parse_primary()?
                    };
                    let span = Span {
                        start: start_tok.span.start,
                        end: block_expr.span().end,
                        line: start_tok.span.line,
                        col: start_tok.span.col,
                    };
                    Ok(Expr::ThreadSpawn(Box::new(block_expr), span))
                }
            }
            TokenKind::Formula => {
                let start_tok = self.advance();
                self.consume(TokenKind::OpenBrace, "expected '{' for formula structure")?;
                let mut mappings = Vec::new();
                while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
                    let key_tok =
                        self.consume(TokenKind::Identifier, "expected formula field key")?;
                    self.consume(TokenKind::Colon, "expected ':' separator")?;
                    let val = self.parse_expr()?;
                    mappings.push((key_tok.lexeme.clone(), val));
                    self.match_token(TokenKind::Comma);
                }
                let end_tok = self.consume(
                    TokenKind::CloseBrace,
                    "expected '}' to close formula structure",
                )?;
                Ok(Expr::Formula(
                    mappings,
                    Span {
                        start: start_tok.span.start,
                        end: end_tok.span.end,
                        line: start_tok.span.line,
                        col: start_tok.span.col,
                    },
                ))
            }
            TokenKind::InterpolatedStringStart => {
                let start_tok = self.advance();
                let mut segments = Vec::new();
                while !self.check(TokenKind::StringEnd) && !self.check(TokenKind::EOF) {
                    if self.check(TokenKind::InterpolatedStringContent) {
                        let tok = self.advance();
                        segments.push(InterpolatedSegment::Text(tok.lexeme.clone()));
                    } else if self.check(TokenKind::InterpolationStart) {
                        self.advance(); // consume '%{'
                        let expr = self.parse_expr()?;
                        segments.push(InterpolatedSegment::Expr(expr));
                        self.consume(
                            TokenKind::InterpolationEnd,
                            "expected '}' to close string interpolation",
                        )?;
                    } else {
                        break;
                    }
                }
                let end_tok =
                    self.consume(TokenKind::StringEnd, "expected ending quote for string")?;
                Ok(Expr::InterpolatedString(
                    segments,
                    Span {
                        start: start_tok.span.start,
                        end: end_tok.span.end,
                        line: start_tok.span.line,
                        col: start_tok.span.col,
                    },
                ))
            }
            TokenKind::OpenBracket => {
                let start_tok = self.advance();
                let mut elements = Vec::new();
                while !self.check(TokenKind::CloseBracket) && !self.check(TokenKind::EOF) {
                    elements.push(self.parse_expr()?);
                    self.match_token(TokenKind::Comma);
                }
                let end_tok =
                    self.consume(TokenKind::CloseBracket, "expected ']' to close list")?;
                Ok(Expr::VectorLiteral(
                    elements,
                    Span {
                        start: start_tok.span.start,
                        end: end_tok.span.end,
                        line: start_tok.span.line,
                        col: start_tok.span.col,
                    },
                ))
            }
            TokenKind::OpenParen => {
                if self.is_closure_lookahead() {
                    return self.parse_closure();
                }
                let start_tok = self.advance();
                let mut expressions = Vec::new();
                while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::EOF) {
                    expressions.push(self.parse_expr()?);
                    self.match_token(TokenKind::Comma);
                }
                let end_tok = self.consume(TokenKind::CloseParen, "expected ')' to close group")?;
                if expressions.len() == 1 {
                    Ok(expressions[0].clone())
                } else {
                    Ok(Expr::Tuple(
                        expressions,
                        Span {
                            start: start_tok.span.start,
                            end: end_tok.span.end,
                            line: start_tok.span.line,
                            col: start_tok.span.col,
                        },
                    ))
                }
            }
            TokenKind::Await => {
                let start_tok = self.advance();
                let expr = self.parse_primary()?;
                let span = Span {
                    start: start_tok.span.start,
                    end: expr.span().end,
                    line: start_tok.span.line,
                    col: start_tok.span.col,
                };
                Ok(Expr::Await(Box::new(expr), span))
            }
            _ => Err(Diagnostic::new_error(
                format!("expected expression, found '{}'", token.lexeme),
                self.filepath.clone(),
                token.span.clone(),
                Some("Failed to parse expression".to_string()),
                Some("Check your expression formatting here".to_string()),
            )),
        }
    }

    fn is_generic_call_lookahead(&self) -> bool {
        let mut idx = self.index;
        while idx < self.tokens.len() {
            let tok = &self.tokens[idx];
            match tok.kind {
                TokenKind::Gt => {
                    if idx + 1 < self.tokens.len() {
                        return self.tokens[idx + 1].kind == TokenKind::OpenParen;
                    }
                    return false;
                }
                TokenKind::Identifier
                | TokenKind::Comma
                | TokenKind::Lt
                | TokenKind::Star
                | TokenKind::SelfUpper
                | TokenKind::Dollar => {
                    idx += 1;
                }
                _ => break,
            }
        }
        false
    }

    fn is_struct_init_lookahead(&self) -> bool {
        if !self.check(TokenKind::OpenBrace) {
            return false;
        }
        if self.index + 1 < self.tokens.len() {
            let next = &self.tokens[self.index + 1];
            if next.kind == TokenKind::Identifier {
                if self.index + 2 < self.tokens.len() {
                    let next2 = &self.tokens[self.index + 2];
                    if next2.kind == TokenKind::Colon {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_closure_lookahead(&self) -> bool {
        let mut idx = self.index;
        // Assume current token is OpenParen
        if idx >= self.tokens.len() || self.tokens[idx].kind != TokenKind::OpenParen {
            return false;
        }
        let mut parens = 0;
        while idx < self.tokens.len() {
            match self.tokens[idx].kind {
                TokenKind::OpenParen => parens += 1,
                TokenKind::CloseParen => {
                    parens -= 1;
                    if parens == 0 {
                        // Look at next token
                        let next_idx = idx + 1;
                        if next_idx < self.tokens.len() {
                            let next_kind = &self.tokens[next_idx].kind;
                            return *next_kind == TokenKind::OpenBrace
                                || *next_kind == TokenKind::Arrow;
                        }
                        return false;
                    }
                }
                TokenKind::EOF => return false,
                _ => {}
            }
            idx += 1;
        }
        false
    }

    fn parse_closure(&mut self) -> Result<Expr, Diagnostic> {
        let start_tok =
            self.consume(TokenKind::OpenParen, "expected '(' for closure parameters")?;
        let mut params = Vec::new();
        while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::EOF) {
            let mut is_ref = false;
            let mut is_mut = false;

            if self.match_token(TokenKind::Ampersand) {
                is_ref = true;
                if self.match_token(TokenKind::Mut) {
                    is_mut = true;
                }
            }

            let p_name_tok = self.consume(TokenKind::Identifier, "expected parameter name")?;
            let name = p_name_tok.lexeme.clone();

            let mut p_type = "Unknown".to_string();
            if self.match_token(TokenKind::Colon) {
                p_type = self.parse_type()?;
            }

            let mut default_val = None;
            if self.match_token(TokenKind::Equal) {
                default_val = Some(self.parse_expr()?);
            }

            params.push(Param {
                name,
                type_name: p_type,
                default_val,
                is_ref,
                is_mut,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        self.consume(
            TokenKind::CloseParen,
            "expected ')' to close parameters list",
        )?;

        let mut return_type = None;
        if self.match_token(TokenKind::Arrow) {
            return_type = Some(self.parse_type()?);
        }

        let body = self.parse_block()?;
        let end_span = self.peek().span.clone();

        Ok(Expr::Closure {
            params,
            return_type,
            body,
            span: Span {
                start: start_tok.span.start,
                end: end_span.start,
                line: start_tok.span.line,
                col: start_tok.span.col,
            },
        })
    }

    fn parse_accessors(&mut self, mut expr: Expr) -> Result<Expr, Diagnostic> {
        loop {
            if self.match_token(TokenKind::Dot) {
                let name = self.consume(
                    TokenKind::Identifier,
                    "expected member identifier after '.'",
                )?;
                let span = Span {
                    start: expr.span().start,
                    end: name.span.end,
                    line: expr.span().line,
                    col: expr.span().col,
                };
                expr = Expr::Dot(Box::new(expr), name.lexeme.clone(), span);
            } else if self.check(TokenKind::Lt) && self.is_generic_call_lookahead() {
                self.advance(); // consume '<'
                while !self.check(TokenKind::Gt) && !self.check(TokenKind::EOF) {
                    self.advance();
                }
                self.consume(
                    TokenKind::Gt,
                    "expected '>' to close generic type arguments",
                )?;
            } else if self.match_token(TokenKind::OpenParen) {
                let mut args = Vec::new();
                while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::EOF) {
                    let mut arg_name = None;
                    if self.check(TokenKind::Identifier) {
                        let id = self.peek().lexeme.clone();
                        if self.index + 1 < self.tokens.len()
                            && self.tokens[self.index + 1].kind == TokenKind::Colon
                        {
                            self.advance(); // consume identifier
                            self.advance(); // consume ':'
                            arg_name = Some(id);
                        }
                    }
                    let val = self.parse_expr()?;
                    args.push((arg_name, val));
                    self.match_token(TokenKind::Comma);
                }
                let end_tok = self.consume(
                    TokenKind::CloseParen,
                    "expected ')' to close argument calls",
                )?;
                let span = Span {
                    start: expr.span().start,
                    end: end_tok.span.end,
                    line: expr.span().line,
                    col: expr.span().col,
                };
                expr = Expr::Call(Box::new(expr), args, span);
            } else if self.is_struct_init_lookahead() {
                self.advance(); // consume '{'
                let mut fields = Vec::new();
                while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::EOF) {
                    let field_tok =
                        self.consume(TokenKind::Identifier, "expected struct field name")?;
                    self.consume(TokenKind::Colon, "expected ':' after field name")?;
                    let val = self.parse_expr()?;
                    fields.push((field_tok.lexeme.clone(), val));
                    self.match_token(TokenKind::Comma);
                }
                let end_tok =
                    self.consume(TokenKind::CloseBrace, "expected '}' to close struct init")?;
                let span = Span {
                    start: expr.span().start,
                    end: end_tok.span.end,
                    line: expr.span().line,
                    col: expr.span().col,
                };
                expr = Expr::StructInit(Box::new(expr), fields, span);
            } else {
                break;
            }
        }
        Ok(expr)
    }
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::LetDecl { span, .. } => span.clone(),
            Stmt::ConstDecl { span, .. } => span.clone(),
            Stmt::FuncDecl { span, .. } => span.clone(),
            Stmt::StructDecl { span, .. } => span.clone(),
            Stmt::EnumDecl { span, .. } => span.clone(),
            Stmt::TraitDecl { span, .. } => span.clone(),
            Stmt::ImplDecl { span, .. } => span.clone(),
            Stmt::ImportDecl { span, .. } => span.clone(),
            Stmt::ExportDecl(_, span) => span.clone(),
            Stmt::ExprStmt(e) => e.span(),
            Stmt::IfStmt { span, .. } => span.clone(),
            Stmt::ForStmt { span, .. } => span.clone(),
            Stmt::WhileStmt { span, .. } => span.clone(),
            Stmt::LoopStmt { span, .. } => span.clone(),
            Stmt::MatchStmt { span, .. } => span.clone(),
            Stmt::ReturnStmt(_, span) => span.clone(),
            Stmt::DeferStmt(_, span) => span.clone(),
            Stmt::Break(s) => s.clone(),
            Stmt::Continue(s) => s.clone(),
            Stmt::PluginDecl { span, .. } => span.clone(),
            Stmt::AnnotationDecl { span, .. } => span.clone(),
        }
    }
}

pub fn is_test_annotation(name: &str) -> bool {
    matches!(
        name,
        "Test"
            | "Setup"
            | "Cleanup"
            | "BeforeAll"
            | "AfterAll"
            | "Benchmark"
            | "Ignore"
            | "Only"
            | "Parameterized"
            | "ExpectPanic"
    )
}

pub fn is_test_statement(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::FuncDecl { annotations, .. } => {
            annotations.iter().any(|a| is_test_annotation(&a.name))
        }
        _ => false,
    }
}
