pub use pcf_ast as ast;
pub use pcf_diagnostics as diagnostics;
pub use pcf_lexer as lexer;
pub use pcf_parser as parser;
pub use pcf_runtime as runtime;
pub use pcf_stdlib as stdlib;

mod pipeline;

pub fn parse(source: &str) -> Result<ast::Program, Error> {
    let parse_result = pipeline::run_parse(source);
    if !parse_result.diagnostics.is_empty() {
        return Err(Error {
            message: format_diagnostics(&parse_result.diagnostics),
        });
    }

    parse_result.program.ok_or_else(|| Error {
        message: "parser produced no program".to_string(),
    })
}

pub fn check(source: &str) -> Result<CheckResult, Error> {
    let (diagnostics, outputs) = pipeline::run_check(source);
    Ok(CheckResult {
        diagnostics,
        outputs,
    })
}

pub fn execute(source: &str) -> Result<runtime::Value, Error> {
    match pipeline::run_execute(source) {
        Ok(value) => Ok(value),
        Err(message) => Err(Error { message }),
    }
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

    #[test]
    fn complete_language_fixture_checks_without_errors() {
        let source = include_str!("../../../tests/fixtures/language_complete.pcf");
        let result = check(source).expect("check result");
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == diagnostics::Severity::Error),
            "{:#?}",
            result.diagnostics
        );
        assert_eq!(
            result
                .outputs
                .iter()
                .map(|output| output.content.as_str())
                .collect::<Vec<_>>(),
            vec!["PCF language fixture"]
        );
    }

    #[test]
    fn reports_unknown_names_and_invalid_methods_with_stable_codes() {
        let result =
            check("let answer = missing;\nr_process \"/x\" FROB {}\n").expect("check result");
        let codes: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.0)
            .collect();
        assert!(codes.contains(&"PCF2003"));
        assert!(codes.contains(&"PCF1005"));
    }

    #[test]
    fn validates_processing_route_constraints() {
        let result = check("allow_methods { GET, GET };\nblock_methods { GET };\nr_process \"bad path\" GET {}\nr_process \"/same\" GET {}\nr_process \"/same\" GET {}\n").expect("check result");
        let codes: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.0)
            .collect();
        assert!(codes.contains(&"PCF4002"));
        assert!(codes.contains(&"PCF4003"));
        assert!(codes.contains(&"PCF4004"));
        assert!(codes.contains(&"PCF4005"));
    }

    #[test]
    fn analyzer_reports_unused_and_unreachable_code_as_warnings() {
        let result = check("import std::json;\nimport std::json;\nfn f(unused_param: int) { let unused_local = 1; return; output \"dead\"; }\n").expect("check result");
        let warnings: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == diagnostics::Severity::Warning)
            .map(|diagnostic| diagnostic.code.0)
            .collect();
        assert!(warnings.contains(&"PCF3004"));
        assert!(warnings.contains(&"PCF3005"));
        assert!(warnings.contains(&"PCF3006"));
        assert!(warnings.contains(&"PCF3007"));
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == diagnostics::Severity::Error)
        );
    }

    #[test]
    fn malformed_syntax_returns_parser_diagnostics_without_panicking() {
        for source in [
            "fn broken() {",
            "import std::;",
            "let value = ;",
            "fn broken()",
            "else {}",
            "fn bad(value: ) {}",
        ] {
            let result = check(source).expect("check result");
            assert!(
                result
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code.0.starts_with("PCF10")),
                "source: {source}; diagnostics: {:#?}",
                result.diagnostics
            );
        }
    }
}
