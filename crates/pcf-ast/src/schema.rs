use pcf_span::Span;

use crate::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaDeclaration {
    pub name: Identifier,
    pub span: Span,
}
