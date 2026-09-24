use std::collections::HashSet;

use pcf_ast::{Item, Program};
use pcf_diagnostics::{Diagnostic, DiagnosticCode, Label, Severity};

#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub struct Validator;

impl Validator {
    pub fn validate(&self, program: &Program) -> ValidationResult {
        let mut diagnostics = Vec::new();
        let mut seen = HashSet::new();

        for item in &program.items {
            match item {
                Item::Function(function) => {
                    let mut parameters = HashSet::new();
                    for parameter in &function.parameters {
                        if !parameters.insert(parameter.name.name.clone()) {
                            diagnostics.push(duplicate_parameter_diagnostic(&parameter.name.name));
                        }
                    }
                }
                Item::Variable(variable)
                    if variable.statement.value.is_none()
                        && !seen.insert(variable.statement.name.name.clone()) =>
                {
                    diagnostics.push(duplicate_parameter_diagnostic(
                        &variable.statement.name.name,
                    ));
                }
                _ => {}
            }
        }

        ValidationResult { diagnostics }
    }
}

fn duplicate_parameter_diagnostic(name: &str) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        code: DiagnosticCode("PCF3001"),
        message: format!("duplicate parameter or declaration `{name}`"),
        labels: vec![Label {
            span: pcf_span::Span::default(),
            message: Some("this symbol repeats in the local scope".to_string()),
            primary: true,
        }],
        notes: vec!["rename one of the declarations to preserve a unique local name.".to_string()],
    }
}
