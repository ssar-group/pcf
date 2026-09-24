pub use pcf_ast as ast;
pub use pcf_diagnostics as diagnostics;
pub use pcf_lexer as lexer;
pub use pcf_parser as parser;
pub use pcf_runtime as runtime;
pub use pcf_stdlib as stdlib;

pub fn parse(source: &str) -> Result<ast::Program, Error> {
    let lex_result = lexer::lex(source);
    let parse_result = parser::parse(&lex_result.tokens);
    let mut diagnostics = lex_result.diagnostics;
    diagnostics.extend(parse_result.diagnostics);

    if !diagnostics.is_empty() {
        return Err(Error {
            message: format_diagnostics(&diagnostics),
        });
    }

    parse_result.program.ok_or_else(|| Error {
        message: "parser produced no program".to_string(),
    })
}

pub fn check(source: &str) -> Result<CheckResult, Error> {
    let lex_result = lexer::lex(source);
    let parse_result = parser::parse(&lex_result.tokens);

    let mut diagnostics = lex_result.diagnostics;
    diagnostics.extend(parse_result.diagnostics.clone());

    if let Some(program) = parse_result.program.as_ref() {
        collect_invalid_output_diagnostics(program, &mut diagnostics);
    }

    let outputs = parse_result
        .program
        .as_ref()
        .map(collect_outputs)
        .unwrap_or_default();

    Ok(CheckResult {
        diagnostics,
        outputs,
    })
}

pub fn execute(source: &str) -> Result<runtime::Value, Error> {
    let lex_result = lexer::lex(source);
    let parse_result = parser::parse(&lex_result.tokens);
    let mut diagnostics = lex_result.diagnostics;
    diagnostics.extend(parse_result.diagnostics);

    if !diagnostics.is_empty() {
        return Err(Error {
            message: format_diagnostics(&diagnostics),
        });
    }

    let program = parse_result.program.ok_or_else(|| Error {
        message: "parser produced no program".to_string(),
    })?;

    let mut runtime = runtime::Runtime::default();
    let mut context = runtime::RuntimeContext {
        environment: &mut runtime.environment,
        registry: &runtime.registry,
        permissions: &runtime.permissions,
        capabilities: &runtime.capabilities,
    };

    let mut evaluator = runtime::Evaluator;
    evaluator
        .evaluate_program(&program, &mut context)
        .map_err(|error| Error {
            message: error.message,
        })
}

fn format_diagnostics(diagnostics: &[diagnostics::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.code.0, diagnostic.message))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Default)]
pub struct CheckResult {
    pub diagnostics: Vec<diagnostics::Diagnostic>,
    pub outputs: Vec<CheckOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckOutput {
    pub content: String,
    pub span: pcf_span::Span,
}

fn collect_invalid_output_diagnostics(
    program: &ast::Program,
    diagnostics: &mut Vec<diagnostics::Diagnostic>,
) {
    for item in &program.items {
        collect_invalid_output_diagnostics_from_item(item, diagnostics);
    }
}

fn collect_invalid_output_diagnostics_from_item(
    item: &ast::Item,
    diagnostics: &mut Vec<diagnostics::Diagnostic>,
) {
    match item {
        ast::Item::Statement(ast::Statement::Output(output)) => {
            if !matches!(output.value.value, ast::Literal::String(_)) {
                diagnostics.push(diagnostics::Diagnostic {
                    severity: diagnostics::Severity::Error,
                    code: diagnostics::DiagnosticCode("PCF1001"),
                    message: "output requires a string literal".to_string(),
                    labels: vec![diagnostics::Label {
                        span: output.span,
                        message: Some("only string literals are valid static output".to_string()),
                        primary: true,
                    }],
                    notes: vec![
                        "static output is limited to string literals in the current grammar."
                            .to_string(),
                    ],
                });
            }
        }
        ast::Item::Statement(ast::Statement::Block(block)) => {
            for statement in &block.statements {
                collect_invalid_output_diagnostics_from_statement(statement, diagnostics);
            }
        }
        ast::Item::Function(function) => {
            for statement in &function.body {
                collect_invalid_output_diagnostics_from_statement(statement, diagnostics);
            }
        }
        _ => {}
    }
}

fn collect_invalid_output_diagnostics_from_statement(
    statement: &ast::Statement,
    diagnostics: &mut Vec<diagnostics::Diagnostic>,
) {
    match statement {
        ast::Statement::Output(output) => {
            if !matches!(output.value.value, ast::Literal::String(_)) {
                diagnostics.push(diagnostics::Diagnostic {
                    severity: diagnostics::Severity::Error,
                    code: diagnostics::DiagnosticCode("PCF1001"),
                    message: "output requires a string literal".to_string(),
                    labels: vec![diagnostics::Label {
                        span: output.span,
                        message: Some("only string literals are valid static output".to_string()),
                        primary: true,
                    }],
                    notes: vec![
                        "static output is limited to string literals in the current grammar."
                            .to_string(),
                    ],
                });
            }
        }
        ast::Statement::Block(block) => {
            for nested in &block.statements {
                collect_invalid_output_diagnostics_from_statement(nested, diagnostics);
            }
        }
        _ => {}
    }
}

fn collect_outputs(program: &ast::Program) -> Vec<CheckOutput> {
    let mut outputs = Vec::new();

    for item in &program.items {
        collect_outputs_from_item(item, &mut outputs);
    }

    outputs
}

fn collect_outputs_from_item(item: &ast::Item, outputs: &mut Vec<CheckOutput>) {
    match item {
        ast::Item::Statement(statement) => collect_outputs_from_statement(statement, outputs),
        ast::Item::Function(function) => {
            for statement in &function.body {
                collect_outputs_from_statement(statement, outputs);
            }
        }
        _ => {}
    }
}

fn collect_outputs_from_statement(statement: &ast::Statement, outputs: &mut Vec<CheckOutput>) {
    match statement {
        ast::Statement::Output(output) => {
            if let ast::Literal::String(content) = &output.value.value {
                outputs.push(CheckOutput {
                    content: content.clone(),
                    span: output.span,
                });
            }
        }
        ast::Statement::Block(block) => {
            for nested in &block.statements {
                collect_outputs_from_statement(nested, outputs);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_static_outputs_in_order() {
        let result = check("output \"hello\"\noutput \"world\"\n").expect("check result");
        let contents: Vec<_> = result
            .outputs
            .into_iter()
            .map(|output| output.content)
            .collect();
        assert_eq!(contents, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn reports_invalid_output_as_diagnostic() {
        let result = check("output 42\n").expect("check result");
        assert!(result.outputs.is_empty());
        assert!(!result.diagnostics.is_empty());
    }
}
