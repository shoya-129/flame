use crate::lexer::{Lexer, TokenKind};

pub fn format_code(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    lexer.keep_comments = true;

    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        if tok.kind == TokenKind::EOF {
            break;
        }
        tokens.push(tok);
    }

    let mut multiline_parens = vec![false; tokens.len()];
    let mut paren_stack = Vec::new();

    for (i, tok) in tokens.iter().enumerate() {
        if tok.kind == TokenKind::OpenParen {
            paren_stack.push(i);
        } else if tok.kind == TokenKind::CloseParen {
            if let Some(start_idx) = paren_stack.pop() {
                let mut has_brace = false;
                for j in (start_idx + 1)..i {
                    if tokens[j].kind == TokenKind::OpenBrace {
                        has_brace = true;
                        break;
                    }
                }
                if has_brace {
                    multiline_parens[start_idx] = true;
                    multiline_parens[i] = true;
                }
            }
        }
    }

    let mut last_import_idx = None;
    for (i, tok) in tokens.iter().enumerate() {
        if tok.kind == TokenKind::Import {
            last_import_idx = Some(i);
        }
    }
    let mut last_import_line_end_idx = None;
    if let Some(idx) = last_import_idx {
        for j in idx..tokens.len() {
            if tokens[j].kind == TokenKind::Newline {
                last_import_line_end_idx = Some(j);
                break;
            }
        }
    }

    let mut out = String::new();
    let mut indent_level: usize = 0;
    let mut needs_indent = true;
    let mut last_kind = TokenKind::EOF;
    let mut in_multiline_paren_count: usize = 0;
    let mut empty_line_pending = false;

    let indent_str = |level: usize| -> String {
        "    ".repeat(level)
    };

    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];

        if tok.kind == TokenKind::Newline {
            if last_kind != TokenKind::Newline || empty_line_pending || Some(i) == last_import_line_end_idx {
                while out.ends_with(' ') || out.ends_with('\t') {
                    out.pop();
                }
                
                if Some(i) == last_import_line_end_idx {
                    if !out.ends_with('\n') {
                        out.push_str("\n\n\n\n");
                    } else {
                        out.push_str("\n\n\n");
                    }
                } else if empty_line_pending {
                    if !out.ends_with("\n\n") {
                        if !out.ends_with('\n') {
                            out.push_str("\n\n");
                        } else {
                            out.push('\n');
                        }
                    }
                } else {
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                }
                
                needs_indent = true;
                empty_line_pending = false;
            }
            last_kind = TokenKind::Newline;
            i += 1;
            continue;
        }

        if tok.kind == TokenKind::CloseBrace {
            indent_level = indent_level.saturating_sub(1);
            if !out.ends_with('\n') {
                out.push('\n');
            }
            needs_indent = true;
        } else if tok.kind == TokenKind::CloseParen && multiline_parens[i] {
            indent_level = indent_level.saturating_sub(1);
            in_multiline_paren_count = in_multiline_paren_count.saturating_sub(1);
            if !out.ends_with('\n') {
                out.push('\n');
            }
            needs_indent = true;
        }

        if needs_indent {
            out.push_str(&indent_str(indent_level));
            needs_indent = false;
        }

        // Spacing before
        match tok.kind {
            TokenKind::OpenBrace | TokenKind::Equal | TokenKind::EqualEqual | TokenKind::ExclamationEqual | TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash | TokenKind::Percent | TokenKind::Le | TokenKind::Ge | TokenKind::Arrow | TokenKind::FatArrow => {
                if !out.ends_with(' ') && !out.ends_with('\n') {
                    out.push(' ');
                }
            }
            _ => {
                let is_last_ident_like = matches!(
                    last_kind,
                    TokenKind::Identifier | TokenKind::Annotation | TokenKind::StringLiteral | TokenKind::IntLiteral | TokenKind::FloatLiteral | TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace | TokenKind::True | TokenKind::False
                );
                let is_current_ident_like = matches!(
                    tok.kind,
                    TokenKind::Identifier | TokenKind::Let | TokenKind::Const | TokenKind::Fn | TokenKind::Struct | TokenKind::Enum | TokenKind::Trait | TokenKind::Impl | TokenKind::Export | TokenKind::Import | TokenKind::Return | TokenKind::Mut | TokenKind::In | TokenKind::As | TokenKind::Match | TokenKind::If | TokenKind::Else | TokenKind::For | TokenKind::While | TokenKind::Loop | TokenKind::Yield | TokenKind::Defer | TokenKind::Async | TokenKind::Await | TokenKind::Thread | TokenKind::Formula | TokenKind::Annotation | TokenKind::Type | TokenKind::Where | TokenKind::True | TokenKind::False
                );

                let is_last_keyword = matches!(
                    last_kind,
                    TokenKind::Let | TokenKind::Const | TokenKind::Fn | TokenKind::Struct | TokenKind::Enum | TokenKind::Trait | TokenKind::Impl | TokenKind::Export | TokenKind::Import | TokenKind::Return | TokenKind::Mut | TokenKind::In | TokenKind::As | TokenKind::Match | TokenKind::If | TokenKind::Else | TokenKind::For | TokenKind::While | TokenKind::Loop | TokenKind::Yield | TokenKind::Defer | TokenKind::Async | TokenKind::Await | TokenKind::Thread | TokenKind::Formula | TokenKind::Annotation | TokenKind::Type | TokenKind::Where | TokenKind::Comma | TokenKind::Colon | TokenKind::Equal | TokenKind::EqualEqual | TokenKind::ExclamationEqual | TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash | TokenKind::Percent | TokenKind::Le | TokenKind::Ge | TokenKind::Arrow | TokenKind::FatArrow
                );

                if tok.kind != TokenKind::Dot && last_kind != TokenKind::Dot && !out.ends_with('.') {
                    if (is_last_ident_like && is_current_ident_like) || is_last_keyword {
                        // Exception: do not add a space before OpenParen if the last token was a keyword that acts like a function (e.g. type())
                        let skip_space = tok.kind == TokenKind::OpenParen && matches!(last_kind, TokenKind::Type | TokenKind::Identifier);
                        if !skip_space && !out.ends_with(' ') && !out.ends_with('\n') {
                            out.push(' ');
                        }
                    }
                }
            }
        }

        let original_text = &source[tok.span.start..tok.span.end];
        out.push_str(original_text);

        // Check if this token requires an empty line after it
        if tok.kind == TokenKind::CloseBrace {
            // Function or block finished, add empty line gap unless followed by else
            let mut next_is_else = false;
            let mut j = i + 1;
            while j < tokens.len() && tokens[j].kind == TokenKind::Newline {
                j += 1;
            }
            if j < tokens.len() && (tokens[j].kind == TokenKind::Else || tokens[j].kind == TokenKind::CloseParen) {
                next_is_else = true;
            }
            if !next_is_else {
                empty_line_pending = true;
            }
        }
        if tok.kind == TokenKind::Let || (tok.kind == TokenKind::Identifier && (original_text == "print" || original_text == "println")) {
            let is_start = i == 0 || matches!(tokens[i-1].kind, TokenKind::Newline | TokenKind::OpenBrace);
            if is_start {
                let mut next_start = None;
                let mut j = i + 1;
                while j < tokens.len() {
                    if tokens[j].kind == TokenKind::Newline {
                        let mut k = j + 1;
                        while k < tokens.len() && tokens[k].kind == TokenKind::Newline {
                            k += 1;
                        }
                        if k < tokens.len() && tokens[k].kind != TokenKind::CloseBrace && tokens[k].kind != TokenKind::CloseParen {
                            next_start = Some(&tokens[k]);
                        }
                        break;
                    }
                    j += 1;
                }
                if let Some(nst) = next_start {
                    let nst_text = &source[nst.span.start..nst.span.end];
                    let same_group = if tok.kind == TokenKind::Let {
                        nst.kind == TokenKind::Let
                    } else {
                        nst.kind == TokenKind::Identifier && (nst_text == "print" || nst_text == "println")
                    };
                    if !same_group {
                        empty_line_pending = true;
                    }
                } else {
                    empty_line_pending = true;
                }
            }
        }

        last_kind = tok.kind.clone();

        // Spacing/newlines after
        match tok.kind {
            TokenKind::OpenBrace => {
                indent_level += 1;
                out.push('\n');
                needs_indent = true;
            }
            TokenKind::OpenParen => {
                if multiline_parens[i] {
                    indent_level += 1;
                    in_multiline_paren_count += 1;
                    out.push('\n');
                    needs_indent = true;
                }
            }
            TokenKind::Comma => {
                if !out.ends_with(' ') {
                    out.push(' ');
                }
                if in_multiline_paren_count > 0 {
                    out.push('\n');
                    needs_indent = true;
                }
            }
            TokenKind::Comment => {
                out.push('\n');
                needs_indent = true;
            }
            _ => {}
        }
        
        i += 1;
    }

    out = out.trim_end().to_string();
    out.push('\n');
    out
}
