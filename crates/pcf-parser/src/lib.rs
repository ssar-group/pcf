mod cursor;
mod parser;
mod recovery;

pub use parser::{Parser, parse};
pub use recovery::RecoveryState;

#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    pub program: Option<pcf_ast::Program>,
    pub diagnostics: Vec<pcf_diagnostics::Diagnostic>,
}
