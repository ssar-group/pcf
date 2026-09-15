use pcf_ast::Program;

use crate::{ResolveError, ResolvedModule};

#[derive(Debug, Clone, Default)]
pub struct ResolveResult {
    pub module: Option<ResolvedModule>,
}

#[derive(Debug, Default)]
pub struct Resolver;

impl Resolver {
    pub fn resolve_program(&self, _program: &Program) -> Result<ResolveResult, ResolveError> {
        Ok(ResolveResult::default())
    }
}
