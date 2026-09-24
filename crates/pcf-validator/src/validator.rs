use std::collections::HashSet;

use pcf_ast::{Item, Program, Statement};
use pcf_diagnostics::{Diagnostic, DiagnosticCode, Label, Severity};
use pcf_span::Span;

#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub struct Validator;

impl Validator {
    /// Validate program-level invariants that are not strictly grammar-related.
    /// This includes: duplicate function parameters and invalid `output` usage.
    pub fn validate(&self, program: &Program) -> ValidationResult {
        let mut diagnostics = Vec::new();

        for item in &program.items {
            match item {
                Item::Function(function) => {
                    let mut parameters = HashSet::new();
                    for parameter in &function.parameters {
                        if !parameters.insert(parameter.name.name.clone()) {
                            diagnostics.push(duplicate_parameter_diagnostic(
                                &parameter.name.name,
                                parameter.name.span,
                            ));
                        }
                    }
                    // validate statements inside function for output misuse
                    for stmt in &function.body {
                        collect_output_diagnostics_from_statement(stmt, &mut diagnostics);
                    }
                }
                Item::Statement(statement) => {
                    collect_output_diagnostics_from_statement(statement, &mut diagnostics);
                }
                Item::Variable(variable) if variable.statement.name.name.is_empty() => {
                    // Defensive: report invalid declaration when the name is empty
                    diagnostics.push(invalid_declaration_diagnostic(variable.statement.span));
                }
                Item::Variable(_) => {}
                _ => {}
            }
        }

        ValidationResult { diagnostics }
    }
}

fn duplicate_parameter_diagnostic(name: &str, span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        code: DiagnosticCode("PCF3001"),
        message: format!("duplicate parameter `{name}`"),
        labels: vec![Label {
            span,
            message: Some("this parameter name is repeated".to_string()),
            primary: true,
        }],
        notes: vec!["rename one of the parameters to preserve a unique local name.".to_string()],
    }
}

fn invalid_declaration_diagnostic(span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF3003"),
        message: "invalid declaration".to_string(),
        labels: vec![Label {
            span,
            message: Some("declaration has invalid form".to_string()),
            primary: true,
        }],
        notes: Vec::new(),
    }
}

fn collect_output_diagnostics_from_statement(
    statement: &Statement,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Statement::Output(output) => {
            if !matches!(output.value.value, pcf_ast::Literal::String(_)) {
                diagnostics.push(invalid_output_diagnostic(output.span));
            }
        }
        Statement::Block(block) => {
            for nested in &block.statements {
                collect_output_diagnostics_from_statement(nested, diagnostics);
            }
        }
        _ => {}
    }
}

fn invalid_output_diagnostic(span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode("PCF1001"),
        message: "output requires a string literal".to_string(),
        labels: vec![Label {
            span,
            message: Some("only string literals are valid static output".to_string()),
            primary: true,
        }],
        notes: vec![
            "static output is limited to string literals in the current grammar.".to_string(),
        ],
    }
}
