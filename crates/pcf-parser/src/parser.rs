use pcf_ast::{Item, Literal, LiteralExpression, OutputStatement, Program, Statement};
use pcf_diagnostics::{Diagnostic, DiagnosticCode, Label, Severity};
use pcf_span::Span;
use pcf_token::{Token, TokenKind};

use crate::ParseResult;

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            position: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn parse_program(&mut self) -> ParseResult {
        if self.tokens.is_empty() {
            return ParseResult {
                program: Some(Program::default()),
                diagnostics: Vec::new(),
            };
        }

        let mut items = Vec::new();
        self.skip_separators();
        while !self.is_at_end() {
            match self.current_kind() {
                Some(TokenKind::Output) => {
                    if let Some(item) = self.parse_output_statement() {
                        items.push(item);
                    }
                }
                Some(TokenKind::Eof) | None => break,
                Some(_) => {
                    if let Some(token) = self.current() {
                        self.diagnostics.push(unexpected_token(token));
                    }
                    self.advance();
                }
            }
            self.skip_separators();
        }

        let span = program_span(&items).unwrap_or_default();
        ParseResult {
            program: Some(Program { items, span }),
            diagnostics: std::mem::take(&mut self.diagnostics),
        }
    }

    fn parse_output_statement(&mut self) -> Option<Item> {
        let output_token = self.advance()?.clone();
        let string_token = self.current()?.clone();

        match string_token.kind {
            TokenKind::String(value) => {
                self.advance();
                let expression = LiteralExpression {
                    value: Literal::String(value),
                    span: string_token.span,
                };
                Some(Item::Statement(Statement::Output(OutputStatement {
                    value: expression,
                    span: Span {
                        start: output_token.span.start,
                        end: string_token.span.end,
                    },
                })))
            }
            _ => {
                self.diagnostics
                    .push(expected_output_string(&output_token, &string_token));
                self.synchronize_output_statement();
                None
            }
        }
    }

    fn synchronize_output_statement(&mut self) {
        while !self.is_at_end() {
            match self.current_kind() {
                Some(
                    TokenKind::Newline
                    | TokenKind::Semicolon
                    | TokenKind::RightBrace
                    | TokenKind::Eof,
                )
                | None => break,
                Some(_) => {
                    self.advance();
                }
            }
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn current_kind(&self) -> Option<&TokenKind> {
        self.current().map(|token| &token.kind)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || matches!(self.current_kind(), Some(TokenKind::Eof))
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.current();
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        token
    }

    fn skip_separators(&mut self) {
        while matches!(
            self.current_kind(),
            Some(TokenKind::Newline | TokenKind::Semicolon)
        ) {
            self.advance();
        }
    }
}

pub fn parse(tokens: &[Token]) -> ParseResult {
    Parser::new(tokens).parse_program()
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
        message: format!("unexpected token {:?}", token.kind),
        labels: vec![Label {
            span: token.span,
            message: Some("this token is not valid here".to_string()),
            primary: true,
        }],
        notes: Vec::new(),
    }
}

fn expected_output_string(output_token: &Token, token: &Token) -> Diagnostic {
    let span = if matches!(token.kind, TokenKind::Eof) {
        output_token.span
    } else {
        token.span
    };
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
            "`output` is a static statement and does not execute runtime expressions yet"
                .to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcf_lexer::lex;

    #[test]
    fn parses_output_statements_in_order() {
        let result = parse(&lex("output \"hello\"\noutput \"world\"\n").tokens);
        assert!(result.diagnostics.is_empty());
        let program = result.program.expect("program");
        assert_eq!(program.items.len(), 2);
        let values: Vec<_> = program
            .items
            .iter()
            .map(|item| match item {
                Item::Statement(Statement::Output(statement)) => statement.value.value.clone(),
                other => panic!("unexpected item: {other:?}"),
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
    fn reports_missing_output_value() {
        let result = parse(&lex("output\n").tokens);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.program.is_some());
    }

    #[test]
    fn reports_non_string_output_value() {
        let result = parse(&lex("output 42\n").tokens);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("string literal"));
    }

    #[test]
    fn accepts_empty_token_input() {
        let result = parse(&[]);
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.program.expect("program").items.len(), 0);
    }
}
