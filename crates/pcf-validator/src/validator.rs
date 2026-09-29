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

        let mut allowed = HashSet::new();
        let mut blocked = HashSet::new();
        for item in &program.items {
            match item {
                Item::AllowMethods(declaration) => {
                    for (method, span) in &declaration.methods {
                        if !allowed.insert(*method) {
                            diagnostics.push(method_diagnostic(
                                "PCF4002",
                                "duplicate HTTP method",
                                *span,
                            ));
                        }
                    }
                }
                Item::BlockMethods(declaration) => {
                    for (method, span) in &declaration.methods {
                        if !blocked.insert(*method) {
                            diagnostics.push(method_diagnostic(
                                "PCF4002",
                                "duplicate HTTP method",
                                *span,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        for item in &program.items {
            if let Item::AllowMethods(declaration) = item {
                for (method, span) in &declaration.methods {
                    if blocked.contains(method) {
                        diagnostics.push(method_diagnostic(
                            "PCF4003",
                            "HTTP method is both allowed and blocked",
                            *span,
                        ));
                    }
                }
            }
        }
        let mut routes = HashSet::new();
        let explicit_allow = !allowed.is_empty();
        for item in &program.items {
            if let Item::Process(route) = item {
                if !route.path.starts_with('/') || route.path.contains(char::is_whitespace) {
                    diagnostics.push(route_diagnostic(route.path_span));
                }
                if !routes.insert((route.path.clone(), route.method)) {
                    diagnostics.push(method_diagnostic(
                        "PCF4004",
                        "duplicate route and HTTP method",
                        route.span,
                    ));
                }
                if blocked.contains(&route.method) {
                    diagnostics.push(method_diagnostic(
                        "PCF4006",
                        "route uses a blocked HTTP method",
                        route.method_span,
                    ));
                } else if explicit_allow && !allowed.contains(&route.method) {
                    diagnostics.push(method_diagnostic(
                        "PCF4007",
                        "route method is not explicitly allowed",
                        route.method_span,
                    ));
                }
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
        Statement::If(flow) => {
            for s in &flow.then_branch.statements {
                collect_output_diagnostics_from_statement(s, diagnostics);
            }
            if let Some(b) = &flow.else_branch {
                for s in &b.statements {
                    collect_output_diagnostics_from_statement(s, diagnostics);
                }
            }
        }
        Statement::While(flow) => {
            for s in &flow.body.statements {
                collect_output_diagnostics_from_statement(s, diagnostics);
            }
        }
        Statement::For(flow) => {
            for s in &flow.body.statements {
                collect_output_diagnostics_from_statement(s, diagnostics);
            }
        }
        _ => {}
    }
}

fn method_diagnostic(code: &'static str, message: &str, span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: DiagnosticCode(code),
        message: message.to_string(),
        labels: vec![Label {
            span,
            message: None,
            primary: true,
        }],
        notes: Vec::new(),
    }
}
fn route_diagnostic(span: Span) -> Diagnostic {
    method_diagnostic("PCF4005", "invalid route path", span)
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
