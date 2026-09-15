use pcf_ast::Program;
use pcf_diagnostics::Diagnostic;

#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub struct Analyzer;

impl Analyzer {
    pub fn analyze(&self, _program: &Program) -> AnalysisResult {
        AnalysisResult::default()
    }
}
