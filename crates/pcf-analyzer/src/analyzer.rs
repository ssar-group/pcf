use std::collections::HashMap;

use pcf_ast::{Item, Program, Statement};
use pcf_diagnostics::{Diagnostic, DiagnosticCode, Label, Severity};
use pcf_span::Span;

#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub struct Analyzer;

impl Analyzer {
    /// Analyze the program and emit diagnostics. Duplicate names are detected
    /// with proper lexical scoping: each block/function introduces a new local
    /// scope. This prevents unrelated top-level and inner-scope names from
    /// colliding.
    pub fn analyze(&self, program: &Program) -> AnalysisResult {
        let mut diagnostics = Vec::new();
        // Stack of scopes; each scope maps a name to its first declaration offset
        let mut scopes: Vec<HashMap<String, usize>> = vec![HashMap::new()];

        // Walk items at top-level
        for item in &program.items {
            match item {
                Item::Function(function) => {
                    // Top-level function name
                    {
                        let current = scopes.last_mut().expect("scope exists");
                        if let Some(previous) =
                            current.insert(function.name.name.clone(), function.name.span.start)
                        {
                            diagnostics.push(duplicate_name_diagnostic(
                                &function.name.name,
                                function.name.span,
                                previous,
                            ));
                        }
                    }
                    // Enter function scope
                    scopes.push(HashMap::new());
                    // parameters are declarations in the function scope
                    for param in &function.parameters {
                        let current = scopes.last_mut().expect("scope exists");
                        if let Some(previous) =
                            current.insert(param.name.name.clone(), param.span.start)
                        {
                            diagnostics.push(duplicate_name_diagnostic(
                                &param.name.name,
                                param.span,
                                previous,
                            ));
                        }
                    }
                    // Analyze body statements recursively
                    analyze_statements(&function.body, &mut scopes, &mut diagnostics);
                    // Leave function scope
                    scopes.pop();
                }
                Item::Variable(variable) => {
                    let current = scopes.last_mut().expect("scope exists");
                    if let Some(previous) = current.insert(
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
                    let current = scopes.last_mut().expect("scope exists");
                    if let Some(previous) =
                        current.insert(module.name.name.clone(), module.name.span.start)
                    {
                        diagnostics.push(duplicate_name_diagnostic(
                            &module.name.name,
                            module.name.span,
                            previous,
                        ));
                    }
                }
                Item::Schema(schema) => {
                    let current = scopes.last_mut().expect("scope exists");
                    if let Some(previous) =
                        current.insert(schema.name.name.clone(), schema.name.span.start)
                    {
                        diagnostics.push(duplicate_name_diagnostic(
                            &schema.name.name,
                            schema.name.span,
                            previous,
                        ));
                    }
                }
                Item::Statement(statement) => {
                    // Top-level statements can introduce nested scopes
                    analyze_statement(statement, &mut scopes, &mut diagnostics);
                }
            }
        }

        AnalysisResult { diagnostics }
    }
}

fn analyze_statements(
    statements: &[Statement],
    scopes: &mut Vec<HashMap<String, usize>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in statements {
        analyze_statement(stmt, scopes, diagnostics);
    }
}

fn analyze_statement(
    statement: &Statement,
    scopes: &mut Vec<HashMap<String, usize>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Statement::Variable(var) => {
            let current = scopes.last_mut().expect("scope exists");
            if let Some(previous) = current.insert(var.name.name.clone(), var.name.span.start) {
                diagnostics.push(duplicate_name_diagnostic(
                    &var.name.name,
                    var.name.span,
                    previous,
                ));
            }
        }
        Statement::Block(block) => {
            // Push a new local scope for the block
            scopes.push(HashMap::new());
            analyze_statements(&block.statements, scopes, diagnostics);
            scopes.pop();
        }
        Statement::Expression(_) | Statement::Output(_) | Statement::Return(_) => {}
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
