use pcf_span::Span;

use crate::{Expression, Identifier, LiteralExpression};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Variable(VariableStatement),
    Output(OutputStatement),
    Expression(ExpressionStatement),
    Block(BlockStatement),
    Return(ReturnStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableStatement {
    pub name: Identifier,
    pub value: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStatement {
    pub expression: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputStatement {
    pub value: LiteralExpression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
    pub span: Span,
}
