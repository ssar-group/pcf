use pcf_span::Span;

use crate::{Identifier, Statement, TypeAnnotation};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FunctionParameter {
    pub name: Identifier,
    pub ty: Option<TypeAnnotation>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub name: Identifier,
    pub parameters: Vec<FunctionParameter>,
    pub body: Vec<Statement>,
    pub span: Span,
}
