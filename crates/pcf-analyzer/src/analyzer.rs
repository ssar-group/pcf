use std::collections::HashMap;

use pcf_ast::{Item, Program};
use pcf_diagnostics::{Diagnostic, DiagnosticCode, Label, Severity};
use pcf_span::Span;

#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub struct Analyzer;

impl Analyzer {
    pub fn analyze(&self, program: &Program) -> AnalysisResult {
        let mut diagnostics = Vec::new();
        let mut seen: HashMap<String, usize> = HashMap::new();

        for item in &program.items {
            match item {
                Item::Function(function) => {
                    if let Some(previous) =
                        seen.insert(function.name.name.clone(), function.name.span.start)
                    {
                        diagnostics.push(duplicate_name_diagnostic(
                            &function.name.name,
                            function.name.span,
                            previous,
                        ));
                    }
                }
                Item::Variable(variable) => {
                    if let Some(previous) = seen.insert(
                        variable.statement.name.name.clone(),
                        variable.statement.name.span.start,
                    ) {
                        diagnostics.push(duplicate_name_diagnostic(
                            &variable.statement.name.name,
                            variable.statement.name.span,
                            previous,
                        ));
                    }
                }
                Item::Import(import) => {
                    if import.path.is_empty() {
                        diagnostics.push(invalid_import_diagnostic(import.span));
                    }
                }
                Item::Module(module) => {
                    if let Some(previous) =
                        seen.insert(module.name.name.clone(), module.name.span.start)
                    {
                        diagnostics.push(duplicate_name_diagnostic(
                            &module.name.name,
                            module.name.span,
                            previous,
                        ));
                    }
                }
                Item::Schema(schema) => {
                    if let Some(previous) =
                        seen.insert(schema.name.name.clone(), schema.name.span.start)
                    {
                        diagnostics.push(duplicate_name_diagnostic(
                            &schema.name.name,
                            schema.name.span,
                            previous,
                        ));
                    }
                }
                Item::Statement(_) => {}
            }
        }

        AnalysisResult { diagnostics }
    }
}

fn duplicate_name_diagnostic(name: &str, span: Span, previous: usize) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        code: DiagnosticCode("PCF2001"),
        message: format!("duplicate symbol `{name}`"),
        labels: vec![
            Label {
                span,
                message: Some("this symbol is declared again".to_string()),
                primary: true,
            },
            Label {
                span: Span { start: previous, end: previous },
                message: Some("previous declaration".to_string()),
                primary: false,
            },
        ],
        notes: vec!["duplicate declarations are allowed to recover, but they usually indicate a bug in the source.".to_string()],
    }
}

fn invalid_import_diagnostic(span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF2002"),
        message: "invalid import declaration".to_string(),
        labels: vec![Label {
            span,
            message: Some("import statements require a non-empty module path".to_string()),
            primary: true,
        }],
        notes: vec!["use `import foo.bar` or `import foo` to reference a module path.".to_string()],
    }
}
