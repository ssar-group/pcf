use pcf_span::Span;

use crate::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportDeclaration {
    pub path: Vec<Identifier>,
    pub span: Span,
}
