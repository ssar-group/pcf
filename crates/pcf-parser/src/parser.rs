use pcf_ast::{
    BinaryExpression, BinaryOperator, BlockStatement, CallExpression, ExportDeclaration,
    Expression, ExpressionStatement, ForStatement, FunctionDeclaration, FunctionParameter,
    HttpMethod, Identifier, IfStatement, ImportDeclaration, Item, Literal, LiteralExpression,
    MemberExpression, MethodDeclaration, ModuleDeclaration, OutputStatement, ProcessDeclaration,
    Program, ReturnStatement, SchemaDeclaration, Statement, TypeAnnotation, UnaryExpression,
    UnaryOperator, VariableDeclaration, VariableStatement, WhileStatement,
};
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
            let before = self.current().map(|token| token.span.start);
            if let Some(item) = self.parse_item() {
                items.push(item);
            }
            if self.current().map(|token| token.span.start) == before && !self.is_at_end() {
                self.advance();
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
            Some(TokenKind::Import) => self.parse_import_declaration().map(Item::Import),
            Some(TokenKind::Module) => self.parse_module_declaration().map(Item::Module),
            Some(TokenKind::Schema) => self.parse_schema_declaration().map(Item::Schema),
            Some(TokenKind::Fn) => self.parse_function_declaration().map(Item::Function),
            Some(TokenKind::Export) => {
                let start = self.current()?.span.start;
                self.advance();
                self.parse_item().map(|item| {
                    let end = item_span(&item).end;
                    Item::Export(ExportDeclaration {
                        declaration: Box::new(item),
                        span: Span { start, end },
                    })
                })
            }
            Some(TokenKind::AllowMethods) => {
                self.parse_method_declaration(true).map(Item::AllowMethods)
            }
            Some(TokenKind::BlockMethods) => {
                self.parse_method_declaration(false).map(Item::BlockMethods)
            }
            Some(TokenKind::RProcess) => self.parse_process_declaration().map(Item::Process),
            Some(TokenKind::Let | TokenKind::Const) => {
                self.parse_variable_declaration().map(Item::Variable)
            }
            Some(
                TokenKind::Output
                | TokenKind::LeftBrace
                | TokenKind::Return
                | TokenKind::Eof
                | TokenKind::Newline
                | TokenKind::Semicolon
                | _,
            ) => self.parse_statement().map(Item::Statement),
            None => None,
        }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.current_kind() {
            Some(TokenKind::Output) => self.parse_output_statement(),
            Some(TokenKind::Let | TokenKind::Const) => self.parse_variable_statement(),
            Some(TokenKind::LeftBrace) => self.parse_block_statement(),
            Some(TokenKind::Return) => self.parse_return_statement(),
            Some(TokenKind::If) => self.parse_if_statement(),
            Some(TokenKind::While) => self.parse_while_statement(),
            Some(TokenKind::For) => self.parse_for_statement(),
            Some(TokenKind::Eof) | None => None,
            Some(_) => {
                let expression = self.parse_expression()?;
                let span = expression_span(&expression);
                if matches!(expression, Expression::Identifier(_)) {
                    self.diagnostics
                        .push(bare_identifier_statement_diagnostic(&expression));
                }
                Some(Statement::Expression(ExpressionStatement {
                    expression,
                    span,
                }))
            }
        }
    }

    fn parse_output_statement(&mut self) -> Option<Statement> {
        let output_token = self.current()?.clone();
        self.advance();

        let value = match self.parse_expression() {
            Some(Expression::Literal(literal)) => {
                if !matches!(literal.value, Literal::String(_)) {
                    self.diagnostics
                        .push(invalid_output_value(&output_token, literal.span));
                    self.recover_statement();
                    return None;
                }
                literal
            }
            Some(expression) => {
                let span = expression_span(&expression);
                self.diagnostics
                    .push(invalid_output_value(&output_token, span));
                self.recover_statement();
                return None;
            }
            None => {
                self.diagnostics
                    .push(expected_output_literal(&output_token));
                return None;
            }
        };

        let end = value.span.end;
        Some(Statement::Output(OutputStatement {
            value,
            span: Span {
                start: output_token.span.start,
                end,
            },
        }))
    }

    fn parse_variable_declaration(&mut self) -> Option<VariableDeclaration> {
        let keyword = self.current()?.clone();
        self.advance();
        let name = self.parse_identifier()?;

        let mut value = None;
        if let Some(equal) = self.consume_if(&TokenKind::Equal) {
            value = self.parse_expression();
            if value.is_none() {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: DiagnosticCode("PCF1008"),
                    message: "expected an expression after `=`".to_string(),
                    labels: vec![Label {
                        span: equal.span,
                        message: Some("initializer is missing".to_string()),
                        primary: true,
                    }],
                    notes: Vec::new(),
                });
            }
        }
        if matches!(keyword.kind, TokenKind::Const) && value.is_none() {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: DiagnosticCode("PCF1009"),
                message: "constant declarations require an initializer".to_string(),
                labels: vec![Label {
                    span: keyword.span,
                    message: Some("add `= expression` to this declaration".to_string()),
                    primary: true,
                }],
                notes: Vec::new(),
            });
        }

        let end = value
            .as_ref()
            .map_or(name.span.end, |expression| expression_span(expression).end);

        Some(VariableDeclaration {
            statement: VariableStatement {
                name,
                value,
                mutable: matches!(keyword.kind, TokenKind::Let),
                span: Span {
                    start: keyword.span.start,
                    end,
                },
            },
        })
    }

    fn parse_variable_statement(&mut self) -> Option<Statement> {
        let declaration = self.parse_variable_declaration()?;
        Some(Statement::Variable(declaration.statement))
    }

    fn parse_function_declaration(&mut self) -> Option<FunctionDeclaration> {
        let fn_token = self.current()?.clone();
        self.advance();
        let name = self.parse_identifier()?;

        if self.expect(&TokenKind::LeftParen).is_none() {
            self.diagnostics
                .push(missing_close_token(&fn_token, &TokenKind::LeftParen));
            return None;
        }

        let mut parameters = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                let param_name = self.parse_identifier()?;
                let mut ty = None;
                if self.consume_if(&TokenKind::Colon).is_some() {
                    ty = self.parse_type_annotation();
                }
                let param_end = ty
                    .as_ref()
                    .and_then(|_| self.cursor.previous())
                    .map_or(param_name.span.end, |token| token.span.end);
                parameters.push(FunctionParameter {
                    name: param_name.clone(),
                    ty,
                    span: Span {
                        start: param_name.span.start,
                        end: param_end,
                    },
                });
                if self.consume_if(&TokenKind::Comma).is_none() {
                    break;
                }
            }
        }

        let closing_paren = self.expect(&TokenKind::RightParen).or_else(|| {
            self.diagnostics
                .push(missing_close_token(&fn_token, &TokenKind::RightParen));
            None
        })?;

        let block = if let Some(Statement::Block(block)) = self.parse_block_statement() {
            block
        } else {
            self.diagnostics.push(missing_block_body(&name));
            BlockStatement {
                statements: Vec::new(),
                span: Span::default(),
            }
        };

        Some(FunctionDeclaration {
            name,
            parameters,
            body: block.statements,
            span: Span {
                start: fn_token.span.start,
                end: block.span.end.max(closing_paren.span.end),
            },
        })
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        let return_token = self.current()?.clone();
        self.advance();

        let value = if self.matches(&[
            TokenKind::Newline,
            TokenKind::Semicolon,
            TokenKind::RightBrace,
            TokenKind::Eof,
        ]) {
            None
        } else {
            self.parse_expression()
        };

        let end = value.as_ref().map_or(return_token.span.end, |expression| {
            expression_span(expression).end
        });

        Some(Statement::Return(ReturnStatement {
            value,
            span: Span {
                start: return_token.span.start,
                end,
            },
        }))
    }

    fn parse_block_statement(&mut self) -> Option<Statement> {
        let opening = self.current()?.clone();
        if self.consume_if(&TokenKind::LeftBrace).is_none() {
            self.diagnostics.push(unexpected_token(&opening));
            return None;
        }

        let mut statements = Vec::new();
        self.skip_separators();

        while !self.is_at_end() && !self.check(&TokenKind::RightBrace) {
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            }
            self.skip_separators();
        }

        let closing = self.expect(&TokenKind::RightBrace).or_else(|| {
            self.diagnostics
                .push(missing_close_token(&opening, &TokenKind::RightBrace));
            None
        });

        Some(Statement::Block(BlockStatement {
            statements,
            span: Span {
                start: opening.span.start,
                end: closing.map_or(opening.span.end, |token| token.span.end),
            },
        }))
    }

    fn parse_import_declaration(&mut self) -> Option<ImportDeclaration> {
        let import_token = self.current()?.clone();
        self.advance();

        let mut path = Vec::new();
        loop {
            let identifier = self.parse_identifier()?;
            path.push(identifier);
            if self.consume_if(&TokenKind::Dot).is_none()
                && self.consume_if(&TokenKind::ColonColon).is_none()
            {
                break;
            }
        }

        let end = path
            .last()
            .map_or(import_token.span.end, |identifier| identifier.span.end);
        Some(ImportDeclaration {
            path,
            span: Span {
                start: import_token.span.start,
                end,
            },
        })
    }

    fn parse_module_declaration(&mut self) -> Option<ModuleDeclaration> {
        let module_token = self.current()?.clone();
        self.advance();
        let name = self.parse_identifier()?;
        let end = if self.check(&TokenKind::LeftBrace) {
            match self.parse_block_statement() {
                Some(Statement::Block(block)) => block.span.end,
                _ => name.span.end,
            }
        } else {
            name.span.end
        };
        Some(ModuleDeclaration {
            name,
            span: Span {
                start: module_token.span.start,
                end,
            },
        })
    }

    fn parse_schema_declaration(&mut self) -> Option<SchemaDeclaration> {
        let schema_token = self.current()?.clone();
        self.advance();
        let name = self.parse_identifier()?;

        let end = if self.check(&TokenKind::LeftBrace) {
            match self.parse_block_statement() {
                Some(Statement::Block(block)) => block.span.end,
                _ => name.span.end,
            }
        } else {
            name.span.end
        };
        Some(SchemaDeclaration {
            name,
            span: Span {
                start: schema_token.span.start,
                end,
            },
        })
    }

    fn parse_type_annotation(&mut self) -> Option<TypeAnnotation> {
        match self.current_kind() {
            Some(TokenKind::Identifier(_)) => {
                let token = self.current()?.clone();
                self.advance();
                let TokenKind::Identifier(name) = token.kind else {
                    return None;
                };
                Some(TypeAnnotation::Named(name))
            }
            Some(TokenKind::LeftBracket) => {
                self.advance();
                let inner = self.parse_type_annotation()?;
                if self.expect(&TokenKind::RightBracket).is_none() {
                    let token = self.current()?.clone();
                    self.diagnostics
                        .push(missing_close_token(&token, &TokenKind::RightBracket));
                }
                Some(TypeAnnotation::Array(Box::new(inner)))
            }
            Some(_) => {
                let token = self.current()?.clone();
                self.diagnostics.push(unexpected_token(&token));
                self.advance();
                None
            }
            None => None,
        }
    }

    fn parse_identifier(&mut self) -> Option<Identifier> {
        let token = self.current()?.clone();
        let TokenKind::Identifier(name) = token.kind.clone() else {
            self.diagnostics.push(unexpected_token(&token));
            self.advance();
            return None;
        };

        self.advance();
        Some(Identifier {
            name,
            span: token.span,
        })
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_precedence(0)
    }

    fn parse_precedence(&mut self, min_precedence: u8) -> Option<Expression> {
        let mut left = self.parse_prefix()?;
        loop {
            if self.consume_if(&TokenKind::Dot).is_some() {
                let property = self.parse_identifier()?;
                let span = Span {
                    start: expression_span(&left).start,
                    end: property.span.end,
                };
                left = Expression::Member(MemberExpression {
                    object: Box::new(left),
                    property,
                    span,
                });
            } else if let Some(open) = self.consume_if(&TokenKind::LeftParen) {
                let mut arguments = Vec::new();
                if !self.check(&TokenKind::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if self.consume_if(&TokenKind::Comma).is_none() {
                            break;
                        }
                    }
                }
                let close = self.expect(&TokenKind::RightParen).unwrap_or_else(|| {
                    self.diagnostics
                        .push(missing_close_token(&open, &TokenKind::RightParen));
                    open.clone()
                });
                let span = Span {
                    start: expression_span(&left).start,
                    end: close.span.end,
                };
                left = Expression::Call(CallExpression {
                    callee: Box::new(left),
                    arguments,
                    span,
                });
            } else {
                break;
            }
        }

        while let Some(kind) = self.current_kind() {
            let Some(precedence) = precedence(&kind) else {
                break;
            };
            if precedence < min_precedence {
                break;
            }

            let operator = self.current()?.clone();
            self.advance();
            let Some(binary_operator) = binary_operator_for(&operator.kind) else {
                self.diagnostics.push(unexpected_token(&operator));
                break;
            };
            let right = self.parse_precedence(precedence + 1)?;
            let span = Span {
                start: expression_span(&left).start,
                end: expression_span(&right).end,
            };
            left = Expression::Binary(BinaryExpression {
                left: Box::new(left),
                operator: binary_operator,
                right: Box::new(right),
                span,
            });
        }

        Some(left)
    }

    fn parse_prefix(&mut self) -> Option<Expression> {
        let token = self.current()?.clone();
        match token.kind {
            TokenKind::Identifier(_) => {
                let identifier = self.parse_identifier()?;
                Some(Expression::Identifier(identifier))
            }
            TokenKind::String(_)
            | TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Null => {
                let literal = self.parse_literal()?;
                Some(Expression::Literal(literal))
            }
            TokenKind::LeftParen => {
                self.advance();
                let inner = self.parse_expression()?;
                self.expect(&TokenKind::RightParen).or_else(|| {
                    self.diagnostics
                        .push(missing_close_token(&token, &TokenKind::RightParen));
                    None
                });
                Some(Expression::Group(Box::new(inner)))
            }
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_precedence(12)?;
                let span = Span {
                    start: token.span.start,
                    end: expression_span(&operand).end,
                };
                Some(Expression::Unary(UnaryExpression {
                    operator: UnaryOperator::Negate,
                    operand: Box::new(operand),
                    span,
                }))
            }
            TokenKind::Bang => {
                self.advance();
                let operand = self.parse_precedence(12)?;
                let span = Span {
                    start: token.span.start,
                    end: expression_span(&operand).end,
                };
                Some(Expression::Unary(UnaryExpression {
                    operator: UnaryOperator::Not,
                    operand: Box::new(operand),
                    span,
                }))
            }
            TokenKind::Eof | TokenKind::Newline | TokenKind::Semicolon => None,
            _ => {
                self.diagnostics.push(unexpected_token(&token));
                self.advance();
                None
            }
        }
    }

    fn parse_method_declaration(&mut self, allow: bool) -> Option<MethodDeclaration> {
        let start = self.current()?.span.start;
        self.advance();
        let Some(opening) = self.expect(&TokenKind::LeftBrace) else {
            if let Some(token) = self.current().cloned() {
                self.diagnostics.push(unexpected_token(&token));
            }
            return None;
        };
        let mut methods = Vec::new();
        self.skip_separators();
        while !self.is_at_end() && !self.check(&TokenKind::RightBrace) {
            let token = self.current()?.clone();
            if let TokenKind::Identifier(name) = &token.kind {
                if let Some(method) = HttpMethod::parse(name) {
                    methods.push((method, token.span));
                } else {
                    self.diagnostics.push(unknown_method(&token));
                }
                self.advance();
            } else {
                self.diagnostics.push(unexpected_token(&token));
                self.advance();
            }
            if self.consume_if(&TokenKind::Comma).is_none() {
                self.skip_separators();
                if !self.check(&TokenKind::RightBrace)
                    && !self.matches(&[TokenKind::Newline, TokenKind::Semicolon])
                {
                    self.diagnostics.push(unexpected_token(self.current()?));
                }
            }
            self.skip_separators();
        }
        let closing = self
            .expect(&TokenKind::RightBrace)
            .unwrap_or(opening.clone());
        let span = Span {
            start,
            end: closing.span.end,
        };
        let _ = allow;
        Some(MethodDeclaration { methods, span })
    }

    fn parse_process_declaration(&mut self) -> Option<ProcessDeclaration> {
        let start = self.current()?.span.start;
        self.advance();
        let path_token = self.current()?.clone();
        let TokenKind::String(path) = path_token.kind.clone() else {
            self.diagnostics.push(invalid_route(&path_token));
            return None;
        };
        self.advance();
        let method_token = self.current()?.clone();
        let method = match &method_token.kind {
            TokenKind::Identifier(name) => HttpMethod::parse(name),
            _ => None,
        };
        let Some(method) = method else {
            self.diagnostics.push(unknown_method(&method_token));
            return None;
        };
        self.advance();
        let Some(Statement::Block(block)) = self.parse_block_statement() else {
            self.diagnostics.push(missing_route_body(method_token.span));
            return None;
        };
        Some(ProcessDeclaration {
            path,
            path_span: path_token.span,
            method,
            method_span: method_token.span,
            body: block.statements,
            span: Span {
                start,
                end: block.span.end,
            },
        })
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        let token = self.current()?.clone();
        self.advance();
        let condition = self.parse_expression()?;
        let Some(Statement::Block(then_branch)) = self.parse_block_statement() else {
            self.diagnostics.push(missing_block_at(token.span));
            return None;
        };
        let else_branch = if self.consume_if(&TokenKind::Else).is_some() {
            if let Some(Statement::Block(block)) = self.parse_block_statement() {
                Some(block)
            } else {
                self.diagnostics.push(missing_block_at(token.span));
                None
            }
        } else {
            None
        };
        let end = else_branch
            .as_ref()
            .map_or(then_branch.span.end, |b| b.span.end);
        Some(Statement::If(IfStatement {
            condition,
            then_branch,
            else_branch,
            span: Span {
                start: token.span.start,
                end,
            },
        }))
    }
    fn parse_while_statement(&mut self) -> Option<Statement> {
        let token = self.current()?.clone();
        self.advance();
        let condition = self.parse_expression()?;
        let Some(Statement::Block(body)) = self.parse_block_statement() else {
            self.diagnostics.push(missing_block_at(token.span));
            return None;
        };
        let end = body.span.end;
        Some(Statement::While(WhileStatement {
            condition,
            body,
            span: Span {
                start: token.span.start,
                end,
            },
        }))
    }
    fn parse_for_statement(&mut self) -> Option<Statement> {
        let token = self.current()?.clone();
        self.advance();
        let item = self.parse_identifier()?;
        if self.expect(&TokenKind::In).is_none() {
            let t = self.current()?.clone();
            self.diagnostics.push(unexpected_token(&t));
            return None;
        }
        let iterable = self.parse_expression()?;
        let Some(Statement::Block(body)) = self.parse_block_statement() else {
            self.diagnostics.push(missing_block_at(token.span));
            return None;
        };
        let end = body.span.end;
        Some(Statement::For(ForStatement {
            item,
            iterable,
            body,
            span: Span {
                start: token.span.start,
                end,
            },
        }))
    }

    fn parse_literal(&mut self) -> Option<LiteralExpression> {
        let token = self.current()?.clone();
        let kind = match token.kind.clone() {
            TokenKind::String(value) => Literal::String(value),
            TokenKind::Integer(value) => Literal::Integer(value),
            TokenKind::Float(value) => Literal::Float(value),
            TokenKind::True => Literal::Boolean(true),
            TokenKind::False => Literal::Boolean(false),
            TokenKind::Null => Literal::Null,
            _ => {
                self.diagnostics.push(unexpected_token(&token));
                self.advance();
                return None;
            }
        };
        self.advance();
        Some(LiteralExpression {
            value: kind,
            span: token.span,
        })
    }

    fn current(&self) -> Option<&Token> {
        self.cursor.current()
    }

    fn current_kind(&self) -> Option<TokenKind> {
        self.cursor.current_kind()
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
    let first_span = item_span(first);
    let last_span = item_span(last);
    Some(Span {
        start: first_span.start,
        end: last_span.end,
    })
}

fn item_span(item: &Item) -> Span {
    match item {
        Item::Import(declaration) => declaration.span,
        Item::Module(declaration) => declaration.span,
        Item::Function(declaration) => declaration.span,
        Item::Variable(declaration) => declaration.statement.span,
        Item::Schema(declaration) => declaration.span,
        Item::AllowMethods(declaration) | Item::BlockMethods(declaration) => declaration.span,
        Item::Process(declaration) => declaration.span,
        Item::Export(export) => export.span,
        Item::Statement(statement) => statement_span(statement),
    }
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::Variable(statement) => statement.span,
        Statement::Output(statement) => statement.span,
        Statement::Expression(statement) => statement.span,
        Statement::Block(statement) => statement.span,
        Statement::Return(statement) => statement.span,
        Statement::If(statement) => statement.span,
        Statement::While(statement) => statement.span,
        Statement::For(statement) => statement.span,
    }
}

fn expression_span(expression: &Expression) -> Span {
    match expression {
        Expression::Literal(literal) => literal.span,
        Expression::Identifier(identifier) => identifier.span,
        Expression::Unary(unary) => unary.span,
        Expression::Binary(binary) => binary.span,
        Expression::Group(group) => expression_span(group),
        Expression::Call(call) => call.span,
        Expression::Member(member) => member.span,
    }
}

fn invalid_output_value(output: &Token, span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1001"),
        message: "expected a literal value after `output`".to_string(),
        labels: vec![
            Label {
                span,
                message: Some("only literal output values are currently allowed".to_string()),
                primary: true,
            },
            Label {
                span: output.span,
                message: Some("`output` expects a literal expression".to_string()),
                primary: false,
            },
        ],
        notes: vec!["the static output grammar currently accepts only literal values.".to_string()],
    }
}

fn expected_output_literal(output: &Token) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1001"),
        message: "expected a literal value after `output`".to_string(),
        labels: vec![Label {
            span: output.span,
            message: Some("missing literal output value".to_string()),
            primary: true,
        }],
        notes: vec!["add a string, number, boolean, or null literal after `output`.".to_string()],
    }
}

fn bare_identifier_statement_diagnostic(expression: &Expression) -> Diagnostic {
    let span = expression_span(expression);
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1004"),
        message: "standalone identifier expressions are not allowed".to_string(),
        labels: vec![Label {
            span,
            message: Some("a bare identifier is not a complete statement".to_string()),
            primary: true,
        }],
        notes: vec!["use a variable declaration, a function call, or an expression with an operator instead.".to_string()],
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

fn missing_close_token(token: &Token, expected: &TokenKind) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1002"),
        message: format!(
            "expected {} to close the preceding construct",
            describe_kind(expected)
        ),
        labels: vec![Label {
            span: token.span,
            message: Some(format!(
                "missing {} before the end of this block",
                describe_kind(expected)
            )),
            primary: true,
        }],
        notes: Vec::new(),
    }
}

fn missing_block_body(name: &Identifier) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1003"),
        message: format!("expected a block body for `{}`", name.name),
        labels: vec![Label {
            span: name.span,
            message: Some("functions require a `{` ... `}` body".to_string()),
            primary: true,
        }],
        notes: Vec::new(),
    }
}

fn unknown_method(token: &Token) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1005"),
        message: "unknown HTTP method".to_string(),
        labels: vec![Label {
            span: token.span,
            message: Some("expected a standard HTTP method".to_string()),
            primary: true,
        }],
        notes: vec![
            "supported methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS, CONNECT, TRACE."
                .to_string(),
        ],
    }
}
fn invalid_route(token: &Token) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1006"),
        message: "route path must be a string literal".to_string(),
        labels: vec![Label {
            span: token.span,
            message: Some("expected a quoted absolute route path".to_string()),
            primary: true,
        }],
        notes: Vec::new(),
    }
}
fn missing_route_body(span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1007"),
        message: "expected a process body".to_string(),
        labels: vec![Label {
            span,
            message: Some("route declarations require a block".to_string()),
            primary: true,
        }],
        notes: Vec::new(),
    }
}
fn missing_block_at(span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1003"),
        message: "expected a block body".to_string(),
        labels: vec![Label {
            span,
            message: Some("control flow requires a `{ ... }` body".to_string()),
            primary: true,
        }],
        notes: Vec::new(),
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
        TokenKind::Schema => "`schema`".to_string(),
        TokenKind::Let => "`let`".to_string(),
        TokenKind::Const => "`const`".to_string(),
        TokenKind::Fn => "`fn`".to_string(),
        TokenKind::Return => "`return`".to_string(),
        TokenKind::If => "`if`".to_string(),
        TokenKind::Else => "`else`".to_string(),
        TokenKind::For => "`for`".to_string(),
        TokenKind::While => "`while`".to_string(),
        TokenKind::In => "`in`".to_string(),
        TokenKind::AllowMethods => "`allow_methods`".to_string(),
        TokenKind::BlockMethods => "`block_methods`".to_string(),
        TokenKind::RProcess => "`r_process`".to_string(),
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
        TokenKind::ColonColon => "`::`".to_string(),
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

fn precedence(kind: &TokenKind) -> Option<u8> {
    Some(match kind {
        TokenKind::Or => 1,
        TokenKind::And => 2,
        TokenKind::EqualEqual | TokenKind::BangEqual => 3,
        TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => 4,
        TokenKind::Plus | TokenKind::Minus => 5,
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 6,
        _ => return None,
    })
}

fn binary_operator_for(kind: &TokenKind) -> Option<BinaryOperator> {
    Some(match kind {
        TokenKind::Plus => BinaryOperator::Add,
        TokenKind::Minus => BinaryOperator::Subtract,
        TokenKind::Star => BinaryOperator::Multiply,
        TokenKind::Slash => BinaryOperator::Divide,
        TokenKind::Percent => BinaryOperator::Modulo,
        TokenKind::EqualEqual => BinaryOperator::Equal,
        TokenKind::BangEqual => BinaryOperator::NotEqual,
        TokenKind::Less => BinaryOperator::Less,
        TokenKind::LessEqual => BinaryOperator::LessEqual,
        TokenKind::Greater => BinaryOperator::Greater,
        TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
        TokenKind::And => BinaryOperator::And,
        TokenKind::Or => BinaryOperator::Or,
        _ => return None,
    })
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
    fn parses_literals_and_expressions() {
        let result = parse(&lex("let answer = 1 + 2 * 3\n").tokens);
        assert!(
            result.diagnostics.is_empty(),
            "diagnostics: {:?}",
            result.diagnostics
        );
        let program = result.program.expect("program");
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Variable(declaration) => {
                assert_eq!(declaration.statement.name.name, "answer");
                assert!(matches!(
                    declaration.statement.value,
                    Some(Expression::Binary(_))
                ));
            }
            _ => panic!("unexpected item"),
        }
    }

    #[test]
    fn parses_function_and_block() {
        let result = parse(&lex("fn greet(name: string) { return name }\n").tokens);
        assert!(
            result.diagnostics.is_empty(),
            "diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.program.as_ref().unwrap().items.len(), 1);
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
        assert!(result.diagnostics[0].message.contains("literal"));
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

    #[test]
    fn function_parameter_span_covers_its_type_annotation_bytes() {
        let source = "fn add(value: int) { return value; }";
        let parsed = parse(&lex(source).tokens);
        let Some(Program { items, .. }) = parsed.program else {
            panic!("program expected");
        };
        let Item::Function(function) = &items[0] else {
            panic!("function expected");
        };
        assert_eq!(
            &source[function.parameters[0].span.start..function.parameters[0].span.end],
            "value: int"
        );
    }
}
