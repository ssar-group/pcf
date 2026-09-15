use pcf_span::Span;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}
