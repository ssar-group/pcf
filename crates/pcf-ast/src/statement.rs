use pcf_span::Span;

use crate::{Expression, Identifier, LiteralExpression};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Variable(VariableStatement),
    Output(OutputStatement),
    Expression(ExpressionStatement),
    Block(BlockStatement),
    Return(ReturnStatement),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableStatement {
    pub name: Identifier,
    pub value: Option<Expression>,
    pub span: Span,
    pub mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: BlockStatement,
    pub else_branch: Option<BlockStatement>,
    pub span: Span,
}
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: BlockStatement,
    pub span: Span,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
    pub item: Identifier,
    pub iterable: Expression,
    pub body: BlockStatement,
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
