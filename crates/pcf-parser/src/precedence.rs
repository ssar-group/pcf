#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Lowest,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Primary,
}
