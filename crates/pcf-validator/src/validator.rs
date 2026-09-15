use pcf_ast::Program;
use pcf_diagnostics::Diagnostic;

#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub struct Validator;

impl Validator {
    pub fn validate(&self, _program: &Program) -> ValidationResult {
        ValidationResult::default()
    }
}
