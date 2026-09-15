#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
    Import,
    Module,
    Function,
    Variable,
    Schema,
}
