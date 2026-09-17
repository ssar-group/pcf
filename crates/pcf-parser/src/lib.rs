mod array;
mod cursor;
mod declaration;
mod error;
mod expression;
mod item;
mod object;
mod parser;
mod precedence;
mod recovery;
mod statement;

pub use parser::{Parser, parse};

#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    pub program: Option<pcf_ast::Program>,
    pub diagnostics: Vec<pcf_diagnostics::Diagnostic>,
}
