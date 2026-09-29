use pcf_span::Span;

use crate::{BinaryOperator, Identifier, Literal, UnaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(LiteralExpression),
    Identifier(Identifier),
    Unary(UnaryExpression),
    Binary(BinaryExpression),
    Group(Box<Expression>),
    Call(CallExpression),
    Member(MemberExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub span: Span,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpression {
    pub object: Box<Expression>,
    pub property: crate::Identifier,
    pub span: Span,
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
