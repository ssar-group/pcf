#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![deny(unsafe_code)]

//! A parser for the PCF language.
//!
//! This crate converts a token stream into an AST while recording recoverable
//! syntax diagnostics. The parser intentionally stays small and focused on
//! validating the grammar that the rest of the PCF toolchain consumes.

mod cursor;
mod parser;
mod recovery;

pub use parser::{Parser, parse};
pub use recovery::RecoveryState;

/// Result of a parse operation.
#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    /// Parsed program, if one could be assembled.
    pub program: Option<pcf_ast::Program>,
    /// Diagnostics collected while parsing.
    pub diagnostics: Vec<pcf_diagnostics::Diagnostic>,
}
