use pcf_span::Span;

use crate::{BinaryOperator, Identifier, Literal, UnaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(LiteralExpression),
    Identifier(Identifier),
    Unary(UnaryExpression),
    Binary(BinaryExpression),
    Group(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralExpression {
    pub value: Literal,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
    pub span: Span,
}
