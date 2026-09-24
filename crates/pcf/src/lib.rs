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
}
