#[derive(serde::Serialize)]
pub struct SemanticToken {
    pub line: usize,
    pub col: usize,
    pub length: usize,
    pub token_type: usize,
    pub token_modifiers: usize,
}

pub fn get_semantic_tokens(source: &str) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    let mut lexer = crate::lexer::Lexer::new(source);

    loop {
        let t = lexer.next_token();
        if t.kind == crate::lexer::TokenKind::EOF {
            break;
        }

        let mut token_type = None;
        let modifiers = 0;

        match t.kind {
            crate::lexer::TokenKind::Comment => {
                token_type = Some(3); // comment
            }
            crate::lexer::TokenKind::StringLiteral
            | crate::lexer::TokenKind::InterpolatedStringContent
            | crate::lexer::TokenKind::StringEnd => {
                token_type = Some(4); // string
            }
            crate::lexer::TokenKind::Annotation => {
                token_type = Some(0); // keyword
            }
            crate::lexer::TokenKind::Fn => {
                token_type = Some(0); // keyword
            }
            crate::lexer::TokenKind::Let
            | crate::lexer::TokenKind::Const
            | crate::lexer::TokenKind::Struct
            | crate::lexer::TokenKind::Enum
            | crate::lexer::TokenKind::Trait
            | crate::lexer::TokenKind::Impl
            | crate::lexer::TokenKind::Export
            | crate::lexer::TokenKind::Import
            | crate::lexer::TokenKind::Mut
            | crate::lexer::TokenKind::As
            | crate::lexer::TokenKind::Type
            | crate::lexer::TokenKind::Where
            | crate::lexer::TokenKind::Formula
            | crate::lexer::TokenKind::If
            | crate::lexer::TokenKind::Else
            | crate::lexer::TokenKind::Match
            | crate::lexer::TokenKind::For
            | crate::lexer::TokenKind::In
            | crate::lexer::TokenKind::While
            | crate::lexer::TokenKind::Loop
            | crate::lexer::TokenKind::Break
            | crate::lexer::TokenKind::Continue
            | crate::lexer::TokenKind::Defer
            | crate::lexer::TokenKind::Return
            | crate::lexer::TokenKind::Yield
            | crate::lexer::TokenKind::Await
            | crate::lexer::TokenKind::Async
            | crate::lexer::TokenKind::Thread
            | crate::lexer::TokenKind::Ampersand2
            | crate::lexer::TokenKind::Pipe2
            | crate::lexer::TokenKind::Exclamation
            | crate::lexer::TokenKind::True
            | crate::lexer::TokenKind::False
            | crate::lexer::TokenKind::Nil => {
                token_type = Some(0); // keyword
            }
            _ => {
                if t.kind == crate::lexer::TokenKind::Identifier
                    && (t.lexeme == "self" || t.lexeme == "Self")
                {
                    token_type = Some(0); // keyword
                }
            }
        }

        if let Some(ty) = token_type {
            tokens.push(SemanticToken {
                line: t.span.line.saturating_sub(1),
                col: t.span.col.saturating_sub(1),
                length: t.span.end.saturating_sub(t.span.start),
                token_type: ty,
                token_modifiers: modifiers,
            });
        }
    }

    tokens
}

