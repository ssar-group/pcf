use pcf_analyzer::Analyzer;
use pcf_ast as ast;
use pcf_diagnostics as diagnostics;
use pcf_lexer as lexer;
use pcf_parser as parser;
use pcf_resolver::Resolver;
use pcf_runtime::{
    CapabilitySet, Evaluator, PermissionSet, Runtime, RuntimeContext, RuntimeRegistry, Value,
};
use pcf_span::Span;
use pcf_validator::Validator;

pub fn run_parse(source: &str) -> parser::ParseResult {
    let lex_result = lexer::lex(source);
    let mut parse_result = parser::parse(&lex_result.tokens);
    let mut diagnostics = lex_result.diagnostics;
    diagnostics.extend(parse_result.diagnostics);
    parse_result.diagnostics = diagnostics;
    parse_result
}

pub fn run_check(source: &str) -> (Vec<diagnostics::Diagnostic>, Vec<crate::CheckOutput>) {
    let parse_result = run_parse(source);
    let mut diagnostics = parse_result.diagnostics.clone();
    let mut outputs = Vec::new();

    if let Some(program) = parse_result.program.as_ref() {
        // Analyzer
        let analyzer = Analyzer::default();
        let analysis = analyzer.analyze(program);
        diagnostics.extend(analysis.diagnostics);

        // Validator
        let validator = Validator::default();
        let validation = validator.validate(program);
        diagnostics.extend(validation.diagnostics);

        // Resolver (errors become diagnostics)
        let resolver = Resolver::default();
        match resolver.resolve_program(program) {
            Ok(_module) => {}
            Err(err) => {
                diagnostics.push(diagnostics::Diagnostic {
                    severity: diagnostics::Severity::Error,
                    code: diagnostics::DiagnosticCode("PCF4001"),
                    message: err.message,
                    labels: Vec::new(),
                    notes: Vec::new(),
                });
            }
        }

        // Collect outputs (static string outputs)
        outputs = collect_outputs(program);
    }

    (diagnostics, outputs)
}

pub fn run_execute(source: &str) -> Result<Value, String> {
    let (diagnostics, _outputs) = run_check(source);
    if diagnostics
        .iter()
        .any(|d| d.severity == diagnostics::Severity::Error)
    {
        return Err(format!("{} diagnostics", diagnostics.len()));
    }

    let parse_result = run_parse(source);
    let program = parse_result
        .program
        .ok_or_else(|| "parser produced no program".to_string())?;

    let mut runtime = Runtime::default();
    let mut context = RuntimeContext {
        environment: &mut runtime.environment,
        registry: &runtime.registry,
        permissions: &runtime.permissions,
        capabilities: &runtime.capabilities,
    };

    let mut evaluator = Evaluator::default();
    evaluator
        .evaluate_program(&program, &mut context)
        .map_err(|e| e.message)
}

fn collect_outputs(program: &ast::Program) -> Vec<crate::CheckOutput> {
    let mut outputs = Vec::new();
    for item in &program.items {
        collect_outputs_from_item(item, &mut outputs);
    }
    outputs
}

fn collect_outputs_from_item(item: &ast::Item, outputs: &mut Vec<crate::CheckOutput>) {
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

fn collect_outputs_from_statement(
    statement: &ast::Statement,
    outputs: &mut Vec<crate::CheckOutput>,
) {
    match statement {
        ast::Statement::Output(output) => {
            if let ast::Literal::String(content) = &output.value.value {
                outputs.push(crate::CheckOutput {
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
