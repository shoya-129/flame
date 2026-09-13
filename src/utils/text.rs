/// Extracts balanced `{ ... }` block starting from `open_brace_pos`.
pub fn extract_balanced_block(source: &str, open_brace_pos: usize) -> Option<&str> {
    let bytes = source.as_bytes();
    if bytes.get(open_brace_pos) != Some(&b'{') {
        return None;
    }
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = b'"';
    let mut escaped = false;
    let start_idx = open_brace_pos + 1;

    for i in open_brace_pos..bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == string_char {
                in_string = false;
            }
            continue;
        }

        if b == b'"' || b == b'\'' {
            in_string = true;
            string_char = b;
            continue;
        }

        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(&source[start_idx..i]);
            }
        }
    }
    None
}

/// Replaces comments and string literals with spaces while preserving newlines and byte positions.
pub fn strip_comments_and_strings(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut lexer = crate::lexer::Lexer::new(source);
    let mut last_idx = 0;

    loop {
        let t = lexer.next_token();
        if t.kind == crate::lexer::TokenKind::EOF {
            if last_idx < source.len() {
                out.push_str(&source[last_idx..]);
            }
            break;
        }

        match t.kind {
            crate::lexer::TokenKind::Comment
            | crate::lexer::TokenKind::StringLiteral
            | crate::lexer::TokenKind::InterpolatedStringContent
            | crate::lexer::TokenKind::StringEnd => {
                if t.span.start > last_idx {
                    out.push_str(&source[last_idx..t.span.start]);
                }
                for ch in source[t.span.start..t.span.end].chars() {
                    if ch == '\n' || ch == '\r' {
                        out.push(ch);
                    } else {
                        for _ in 0..ch.len_utf8() {
                            out.push(' ');
                        }
                    }
                }
                last_idx = t.span.end;
            }
            _ => {}
        }
    }

    out
}
