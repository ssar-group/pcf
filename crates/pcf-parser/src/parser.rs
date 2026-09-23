use pcf_ast::{Item, Literal, LiteralExpression, OutputStatement, Program, Statement};
use pcf_diagnostics::{Diagnostic, DiagnosticCode, Label, Severity};
use pcf_span::Span;
use pcf_token::{Token, TokenKind};

use crate::{ParseResult, cursor::Cursor, recovery::RecoveryState};

/// Parser for converting a token stream into a PCF AST.
#[derive(Debug)]
pub struct Parser<'a> {
    cursor: Cursor<'a>,
    diagnostics: Vec<Diagnostic>,
    recovery: RecoveryState,
}

impl<'a> Parser<'a> {
    /// Creates a parser for a token slice.
    #[must_use]
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            cursor: Cursor::new(tokens),
            diagnostics: Vec::new(),
            recovery: RecoveryState::default(),
        }
    }

    /// Parses the current token stream into a program.
    pub fn parse_program(&mut self) -> ParseResult {
        if self.cursor.current().is_none() {
            return ParseResult {
                program: Some(Program::default()),
                diagnostics: Vec::new(),
            };
        }

        let mut items = Vec::new();
        self.skip_separators();

        while !self.is_at_end() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            }
            self.skip_separators();
        }

        let span = program_span(&items).unwrap_or_default();

        ParseResult {
            program: Some(Program { items, span }),
            diagnostics: std::mem::take(&mut self.diagnostics),
        }
    }

    fn parse_item(&mut self) -> Option<Item> {
        match self.current_kind() {
            Some(TokenKind::Output) => self.parse_output_statement(),
            Some(TokenKind::Eof) | None => None,
            Some(_) => {
                let token = self.current()?.clone();
                self.diagnostics.push(unexpected_token(&token));
                self.advance();
                None
            }
        }
    }

    fn parse_output_statement(&mut self) -> Option<Item> {
        let output_token = self.current()?.clone();
        if self.expect(&TokenKind::Output).is_none() {
            self.diagnostics.push(unexpected_token(&output_token));
            return None;
        }

        let value_token = self.current().cloned();
        if value_token.is_none()
            && self.peek(1).is_some_and(|token| {
                matches!(
                    token.kind,
                    TokenKind::Newline | TokenKind::Semicolon | TokenKind::Eof
                )
            })
        {
            self.diagnostics
                .push(expected_output_string(&output_token, None));
            self.recover_statement();
            return None;
        }

        if let Some(token) = value_token {
            if let TokenKind::String(value) = token.kind {
                self.advance();
                let expression = LiteralExpression {
                    value: Literal::String(value.clone()),
                    span: token.span,
                };
                Some(Item::Statement(Statement::Output(OutputStatement {
                    value: expression,
                    span: Span {
                        start: output_token.span.start,
                        end: token.span.end,
                    },
                })))
            } else {
                self.diagnostics
                    .push(expected_output_string(&output_token, Some(&token)));
                self.recover_statement();
                None
            }
        } else {
            self.diagnostics
                .push(expected_output_string(&output_token, None));
            self.recover_statement();
            None
        }
    }

    fn recover_statement(&mut self) {
        self.recovery.begin();
        while !self.is_at_end() {
            let boundary = self.current_kind().is_some_and(|kind| {
                matches!(
                    kind,
                    TokenKind::Newline
                        | TokenKind::Semicolon
                        | TokenKind::RightBrace
                        | TokenKind::Eof
                )
            });
            if boundary {
                break;
            }
            self.advance();
        }
        self.recovery.end();
    }

    fn current(&self) -> Option<&Token> {
        self.cursor.current()
    }

    fn current_kind(&self) -> Option<TokenKind> {
        self.cursor.current_kind()
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        self.cursor.peek(offset)
    }

    fn advance(&mut self) -> Option<Token> {
        self.cursor.advance()
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.cursor.check(kind)
    }

    fn matches(&self, kinds: &[TokenKind]) -> bool {
        self.cursor.matches(kinds)
    }

    fn expect(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.check(kind) {
            self.advance()
        } else {
            None
        }
    }

    fn consume_if(&mut self, kind: &TokenKind) -> Option<Token> {
        self.cursor.consume_if(kind)
    }

    fn is_at_end(&self) -> bool {
        self.cursor.is_end()
    }

    fn skip_separators(&mut self) {
        while self.matches(&[TokenKind::Newline, TokenKind::Semicolon]) {
            if self.matches(&[TokenKind::Newline]) {
                let _ = self.consume_if(&TokenKind::Newline);
            } else {
                let _ = self.consume_if(&TokenKind::Semicolon);
            }
        }
    }
}

/** Parses a slice of tokens into a program and diagnostics collection */
#[must_use]
pub fn parse(tokens: &[Token]) -> ParseResult {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

fn program_span(items: &[Item]) -> Option<Span> {
    let first = items.first()?;
    let last = items.last()?;
    Some(Span {
        start: item_span(first).start,
        end: item_span(last).end,
    })
}

fn item_span(item: &Item) -> Span {
    match item {
        Item::Statement(Statement::Output(statement)) => statement.span,
        Item::Statement(Statement::Variable(statement)) => statement.span,
        Item::Statement(Statement::Expression(statement)) => statement.span,
        Item::Statement(Statement::Block(statement)) => statement.span,
        Item::Statement(Statement::Return(statement)) => statement.span,
        Item::Import(declaration) => declaration.span,
        Item::Module(declaration) => declaration.span,
        Item::Function(declaration) => declaration.span,
        Item::Variable(declaration) => declaration.statement.span,
        Item::Schema(declaration) => declaration.span,
    }
}

fn unexpected_token(token: &Token) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1000"),
        message: format!("unexpected token {}", describe_kind(&token.kind)),
        labels: vec![Label {
            span: token.span,
            message: Some("this token is not valid here".to_string()),
            primary: true,
        }],
        notes: Vec::new(),
    }
}

fn expected_output_string(output_token: &Token, token: Option<&Token>) -> Diagnostic {
    let span = token.map_or(output_token.span, |token| token.span);
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1001"),
        message: "expected string literal after `output`".to_string(),
        labels: vec![Label {
            span,
            message: Some("only string literals are supported".to_string()),
            primary: true,
        }],
        notes: vec![
            "the current grammar only accepts a static string literal after `output`".to_string(),
        ],
    }
}

fn describe_kind(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Identifier(_) => "an identifier".to_string(),
        TokenKind::String(_) => "a string literal".to_string(),
        TokenKind::Integer(_) => "an integer literal".to_string(),
        TokenKind::Float(_) => "a float literal".to_string(),
        TokenKind::True => "`true`".to_string(),
        TokenKind::False => "`false`".to_string(),
        TokenKind::Null => "`null`".to_string(),
        TokenKind::Output => "`output`".to_string(),
        TokenKind::Import => "`import`".to_string(),
        TokenKind::Export => "`export`".to_string(),
        TokenKind::Module => "`module`".to_string(),
        TokenKind::Let => "`let`".to_string(),
        TokenKind::Const => "`const`".to_string(),
        TokenKind::Fn => "`fn`".to_string(),
        TokenKind::Return => "`return`".to_string(),
        TokenKind::If => "`if`".to_string(),
        TokenKind::Else => "`else`".to_string(),
        TokenKind::For => "`for`".to_string(),
        TokenKind::While => "`while`".to_string(),
        TokenKind::In => "`in`".to_string(),
        TokenKind::LeftBrace => "`{`".to_string(),
        TokenKind::RightBrace => "`}`".to_string(),
        TokenKind::LeftBracket => "`[`".to_string(),
        TokenKind::RightBracket => "`]`".to_string(),
        TokenKind::LeftParen => "`(`".to_string(),
        TokenKind::RightParen => "`)`".to_string(),
        TokenKind::Colon => "`:`".to_string(),
        TokenKind::Semicolon => "`;`".to_string(),
        TokenKind::Comma => "`,`".to_string(),
        TokenKind::Dot => "`.`".to_string(),
        TokenKind::Equal => "`=`".to_string(),
        TokenKind::Plus => "`+`".to_string(),
        TokenKind::Minus => "`-`".to_string(),
        TokenKind::Star => "`*`".to_string(),
        TokenKind::Slash => "`/`".to_string(),
        TokenKind::Percent => "`%`".to_string(),
        TokenKind::EqualEqual => "`==`".to_string(),
        TokenKind::BangEqual => "`!=`".to_string(),
        TokenKind::Less => "`<`".to_string(),
        TokenKind::LessEqual => "`<=`".to_string(),
        TokenKind::Greater => "`>`".to_string(),
        TokenKind::GreaterEqual => "`>=`".to_string(),
        TokenKind::And => "`&&`".to_string(),
        TokenKind::Or => "`||`".to_string(),
        TokenKind::Bang => "`!`".to_string(),
        TokenKind::Newline => "a newline".to_string(),
        TokenKind::Eof => "end of input".to_string(),
        TokenKind::Unknown(ch) => format!("an unknown character `{ch}`"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcf_lexer::lex;

    #[test]
    fn parses_output_statement_in_order() {
        let result = parse(&lex("output \"hello\"\noutput \"world\"\n").tokens);
        assert!(result.diagnostics.is_empty());
        let program = result.program.expect("program");
        assert_eq!(program.items.len(), 2);
        let values: Vec<_> = program
            .items
            .iter()
            .map(|item| match item {
                Item::Statement(Statement::Output(statement)) => statement.value.value.clone(),
                _ => panic!("unexpected item"),
            })
            .collect();
        assert_eq!(
            values,
            vec![
                Literal::String("hello".to_string()),
                Literal::String("world".to_string())
            ]
        );
    }

    #[test]
    fn accepts_empty_input() {
        let result = parse(&[]);
        assert!(result.diagnostics.is_empty());
        assert!(result.program.is_some());
        assert_eq!(result.program.unwrap().items.len(), 0);
    }

    #[test]
    fn accepts_eof_only_input() {
        let result = parse(&[Token {
            kind: TokenKind::Eof,
            span: Span::default(),
        }]);
        assert!(result.diagnostics.is_empty());
        assert!(result.program.is_some());
        assert_eq!(result.program.unwrap().items.len(), 0);
    }

    #[test]
    fn handles_newline_separators() {
        let result = parse(&lex("output \"a\"\noutput \"b\"\n").tokens);
        assert_eq!(result.program.unwrap().items.len(), 2);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn handles_semicolon_separators() {
        let result = parse(&lex("output \"a\";output \"b\";\n").tokens);
        assert_eq!(result.program.unwrap().items.len(), 2);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn reports_missing_output_value() {
        let result = parse(&lex("output\n").tokens);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.program.is_some());
        assert_eq!(result.diagnostics[0].code.0, "PCF1001");
    }

    #[test]
    fn reports_invalid_output_value() {
        let result = parse(&lex("output 42\n").tokens);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("string literal"));
    }

    #[test]
    fn reports_unexpected_token() {
        let result = parse(&lex("@\n").tokens);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("unexpected token"));
    }

    #[test]
    fn recovers_after_invalid_statement() {
        let result = parse(&lex("output 1\noutput \"ok\"\n").tokens);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.program.unwrap().items.len(), 1);
    }

    #[test]
    fn reports_multiple_errors_in_one_input() {
        let result = parse(&lex("output\noutput 7\n").tokens);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn does_not_panic_on_malformed_token_streams() {
        for tokens in [
            Vec::<Token>::new(),
            vec![Token {
                kind: TokenKind::Eof,
                span: Span::default(),
            }],
            vec![Token {
                kind: TokenKind::Output,
                span: Span::default(),
            }],
            vec![
                Token {
                    kind: TokenKind::Output,
                    span: Span::default(),
                },
                Token {
                    kind: TokenKind::Eof,
                    span: Span::default(),
                },
            ],
            vec![Token {
                kind: TokenKind::Newline,
                span: Span::default(),
            }],
            vec![Token {
                kind: TokenKind::Semicolon,
                span: Span::default(),
            }],
            vec![Token {
                kind: TokenKind::RightBrace,
                span: Span::default(),
            }],
        ] {
            let result = parse(&tokens);
            assert!(result.program.is_some());
        }
    }

    #[test]
    fn preserves_source_spans() {
        let source = "output \"hello\"\n";
        let result = parse(&lex(source).tokens);
        let program = result.program.unwrap();
        let span = match &program.items[0] {
            Item::Statement(Statement::Output(statement)) => statement.span,
            _ => panic!("unexpected item"),
        };
        assert_eq!(span.start, 0);
        assert_eq!(span.end, source.len() - 1);
    }
}
