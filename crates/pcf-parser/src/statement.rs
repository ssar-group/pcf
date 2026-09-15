#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementKind {
    Variable,
    Output,
    Expression,
    Return,
    Block,
}
