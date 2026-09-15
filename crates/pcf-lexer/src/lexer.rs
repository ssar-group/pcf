use pcf_diagnostics::{Diagnostic, DiagnosticCode, Label, Severity};
use pcf_span::Span;
use pcf_token::{Token, TokenKind, keyword_kind};

use crate::{
    cursor::Cursor,
    identifier::{is_identifier_continue, is_identifier_start},
    number::{parse_float, parse_number},
    string::unescape_char,
};

#[derive(Debug, Clone, Default)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &str) -> LexResult {
    let mut cursor = Cursor::new(source);
    let mut result = LexResult::default();

    while !cursor.is_eof() {
        let start = cursor.offset();
        let Some(ch) = cursor.peek() else {
            break;
        };

        match ch {
            c if c.is_whitespace() && c != '\n' => {
                let _ = cursor.advance();
            }
            '\n' => {
                let _ = cursor.advance();
                result.tokens.push(Token {
                    kind: TokenKind::Newline,
                    span: Span {
                        start,
                        end: cursor.offset(),
                    },
                });
            }
            '/' if cursor.peek_next() == Some('/') => {
                let _ = cursor.advance();
                let _ = cursor.advance();
                let _ = cursor.consume_while(|c| c != '\n');
            }
            '#' => {
                let _ = cursor.advance();
                let _ = cursor.consume_while(|c| c != '\n');
            }
            '"' => {
                let (token, diagnostic) = lex_string(&mut cursor, start);
                result.tokens.push(token);
                if let Some(diagnostic) = diagnostic {
                    result.diagnostics.push(diagnostic);
                }
            }
            c if c.is_ascii_digit() => {
                let token = lex_number(&mut cursor, start, source);
                result.tokens.push(token);
            }
            c if is_identifier_start(c) => {
                let token = lex_identifier(&mut cursor, start);
                result.tokens.push(token);
            }
            '{' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::LeftBrace),
            '}' => push_single(
                &mut result.tokens,
                &mut cursor,
                start,
                TokenKind::RightBrace,
            ),
            '[' => push_single(
                &mut result.tokens,
                &mut cursor,
                start,
                TokenKind::LeftBracket,
            ),
            ']' => push_single(
                &mut result.tokens,
                &mut cursor,
                start,
                TokenKind::RightBracket,
            ),
            '(' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::LeftParen),
            ')' => push_single(
                &mut result.tokens,
                &mut cursor,
                start,
                TokenKind::RightParen,
            ),
            ':' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Colon),
            ';' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Semicolon),
            ',' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Comma),
            '.' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Dot),
            '+' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Plus),
            '-' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Minus),
            '*' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Star),
            '%' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Percent),
            '!' if cursor.peek_next() == Some('=') => {
                push_double(&mut result.tokens, &mut cursor, start, TokenKind::BangEqual)
            }
            '=' if cursor.peek_next() == Some('=') => push_double(
                &mut result.tokens,
                &mut cursor,
                start,
                TokenKind::EqualEqual,
            ),
            '<' if cursor.peek_next() == Some('=') => {
                push_double(&mut result.tokens, &mut cursor, start, TokenKind::LessEqual)
            }
            '>' if cursor.peek_next() == Some('=') => push_double(
                &mut result.tokens,
                &mut cursor,
                start,
                TokenKind::GreaterEqual,
            ),
            '&' if cursor.peek_next() == Some('&') => {
                push_double(&mut result.tokens, &mut cursor, start, TokenKind::And)
            }
            '|' if cursor.peek_next() == Some('|') => {
                push_double(&mut result.tokens, &mut cursor, start, TokenKind::Or)
            }
            '=' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Equal),
            '<' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Less),
            '>' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Greater),
            '!' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Bang),
            '/' => push_single(&mut result.tokens, &mut cursor, start, TokenKind::Slash),
            other => {
                let _ = cursor.advance();
                result
                    .diagnostics
                    .push(invalid_character(other, start, cursor.offset()));
                result.tokens.push(Token {
                    kind: TokenKind::Unknown(other),
                    span: Span {
                        start,
                        end: cursor.offset(),
                    },
                });
            }
        }
    }

    result.tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span {
            start: source.len(),
            end: source.len(),
        },
    });

    result
}

fn push_single(tokens: &mut Vec<Token>, cursor: &mut Cursor<'_>, start: usize, kind: TokenKind) {
    let _ = cursor.advance();
    tokens.push(Token {
        kind,
        span: Span {
            start,
            end: cursor.offset(),
        },
    });
}

fn push_double(tokens: &mut Vec<Token>, cursor: &mut Cursor<'_>, start: usize, kind: TokenKind) {
    let _ = cursor.advance();
    let _ = cursor.advance();
    tokens.push(Token {
        kind,
        span: Span {
            start,
            end: cursor.offset(),
        },
    });
}

fn lex_identifier(cursor: &mut Cursor<'_>, start: usize) -> Token {
    let ident = cursor.consume_while(is_identifier_continue);
    let kind = keyword_kind(ident).unwrap_or_else(|| TokenKind::Identifier(ident.to_string()));
    Token {
        kind,
        span: Span {
            start,
            end: cursor.offset(),
        },
    }
}

fn lex_number(cursor: &mut Cursor<'_>, start: usize, source: &str) -> Token {
    let mut is_float = false;
    let _ = cursor.consume_while(|c| c.is_ascii_digit());
    if cursor.peek() == Some('.') && cursor.peek_next().is_some_and(|c| c.is_ascii_digit()) {
        is_float = true;
        let _ = cursor.advance();
        let _ = cursor.consume_while(|c| c.is_ascii_digit());
    }
    let text = &source[start..cursor.offset()];
    let kind = if is_float {
        parse_float(text)
            .map(TokenKind::Float)
            .unwrap_or_else(|| TokenKind::Unknown('?'))
    } else {
        parse_number(text)
            .map(TokenKind::Integer)
            .unwrap_or_else(|| TokenKind::Unknown('?'))
    };
    Token {
        kind,
        span: Span {
            start,
            end: cursor.offset(),
        },
    }
}

fn lex_string(cursor: &mut Cursor<'_>, start: usize) -> (Token, Option<Diagnostic>) {
    let _ = cursor.advance();
    let mut value = String::new();
    let mut terminated = false;
    let mut diagnostic = None;

    while let Some(ch) = cursor.peek() {
        match ch {
            '"' => {
                let _ = cursor.advance();
                terminated = true;
                break;
            }
            '\\' => {
                let _ = cursor.advance();
                match cursor.advance() {
                    Some(escape) => {
                        if let Some(resolved) = unescape_char(escape) {
                            value.push(resolved);
                        } else {
                            diagnostic = Some(Diagnostic {
                                severity: Severity::Error,
                                code: DiagnosticCode("PCF0002"),
                                message: format!("invalid string escape `\\{escape}`"),
                                labels: vec![Label {
                                    span: Span {
                                        start,
                                        end: cursor.offset(),
                                    },
                                    message: Some("unknown escape sequence".to_string()),
                                    primary: true,
                                }],
                                notes: Vec::new(),
                            });
                            value.push(escape);
                        }
                    }
                    None => break,
                }
            }
            '\n' => break,
            other => {
                value.push(other);
                let _ = cursor.advance();
            }
        }
    }

    if !terminated {
        let end = cursor.offset();
        return (
            Token {
                kind: TokenKind::String(value),
                span: Span { start, end },
            },
            Some(diagnostic.unwrap_or(Diagnostic {
                severity: Severity::Error,
                code: DiagnosticCode("PCF0003"),
                message: "unterminated string literal".to_string(),
                labels: vec![Label {
                    span: Span { start, end },
                    message: Some("string literal is missing a closing quote".to_string()),
                    primary: true,
                }],
                notes: Vec::new(),
            })),
        );
    }

    (
        Token {
            kind: TokenKind::String(value),
            span: Span {
                start,
                end: cursor.offset(),
            },
        },
        diagnostic,
    )
}

fn invalid_character(ch: char, start: usize, end: usize) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF0001"),
        message: format!("invalid character `{ch}`"),
        labels: vec![Label {
            span: Span { start, end },
            message: Some("this character is not valid here".to_string()),
            primary: true,
        }],
        notes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_basic_module() {
        let result = lex("module project {\n  name = \"PCF\"\n}\n");
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.tokens[0].kind, TokenKind::Module);
        assert_eq!(
            result.tokens[1].kind,
            TokenKind::Identifier("project".to_string())
        );
        assert_eq!(result.tokens[2].kind, TokenKind::LeftBrace);
        assert_eq!(result.tokens[3].kind, TokenKind::Newline);
        assert_eq!(
            result.tokens.last().map(|token| &token.kind),
            Some(&TokenKind::Eof)
        );
    }

    #[test]
    fn lexes_output_keyword() {
        let result = lex("output \"hello\"");
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.tokens[0].kind, TokenKind::Output);
        assert_eq!(result.tokens[1].kind, TokenKind::String("hello".to_string()));
    }

    #[test]
    fn reports_invalid_character() {
        let result = lex("@");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.tokens[0].kind, TokenKind::Unknown('@'));
    }
}
