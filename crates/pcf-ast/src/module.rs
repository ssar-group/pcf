use pcf_span::Span;

use crate::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleDeclaration {
    pub name: Identifier,
    pub span: Span,
}
