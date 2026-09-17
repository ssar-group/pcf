mod ui;

use std::{fs, path::{Path, PathBuf}, time::Instant};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use pcf::{self, diagnostics::Diagnostic, lexer, parser};

use crate::ui::FooterStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus { Success, Failure }

impl ExitStatus {
    pub fn code(self) -> i32 { match self { Self::Success => 0, Self::Failure => 1 } }
}

#[derive(Debug, Parser)]
#[command(name = "pcf", version, about = "PCF command-line interface")]
struct Cli { #[command(subcommand)] command: Commands }

#[derive(Debug, Subcommand)]
enum Commands {
    Inspect(InspectArgs),
    Parse(FileArgs),
    Check(FileArgs),
    Run(FileArgs),
}

#[derive(Debug, Parser)]
struct InspectArgs {
    file: PathBuf,
    #[arg(long)] tokens: bool,
    #[arg(long)] ast: bool,
}

#[derive(Debug, Parser)]
struct FileArgs { file: PathBuf }

pub fn run() -> Result<ExitStatus> {
    match Cli::parse().command {
        Commands::Inspect(args) => inspect(args),
        Commands::Parse(args) => parse_command(args),
        Commands::Check(args) => check_command(args),
        Commands::Run(args) => run_command(args),
    }
}

fn inspect(args: InspectArgs) -> Result<ExitStatus> {
    let started = Instant::now();
    let source = read_source(&args.file)?;
    let lex_result = lexer::lex(&source);
    let parse_result = parser::parse(&lex_result.tokens);
    let tokens_selected = args.tokens || !args.ast;
    let ast_selected = args.ast;
    let file_display = args.file.display().to_string();

    let mut report = new_report("inspect", &file_display, inspect_state(tokens_selected, ast_selected));
    report.section("Diagnostics", |section| {
        let mut diagnostics = lex_result.diagnostics.clone();
        diagnostics.extend(parse_result.diagnostics.clone());
        render_diagnostic_summary(section, &diagnostics);
    });
    report.blank();
    report.section("Output", |section| {
        if tokens_selected {
            render_tokens(section, &lex_result.tokens);
        }

        if ast_selected {
            if tokens_selected {
                section.blank();
            }

            section.section("AST", |ast| {
                match parse_result.program {
                    Some(program) if parse_result.diagnostics.is_empty() => {
                        render_text_lines(ast, format!("{program:#?}"));
                    }
                    Some(program) => {
                        render_text_lines(ast, format!("{program:#?}"));
                        ast.line("parse completed with diagnostics");
                    }
                    None => {
                        ast.line("parse failed: no program produced");
                    }
                }
            });
        }
    });
    report.blank();
    report.section("Next", |section| {
        section.command("parse", command_parse(&args.file));
        section.command("check", command_check(&args.file));
    });

    let exit_status = if has_errors(&lex_result.diagnostics) || has_errors(&parse_result.diagnostics) {
        ExitStatus::Failure
    } else {
        ExitStatus::Success
    };
    print_output(report.finish(footer_status(exit_status), started.elapsed()));
    Ok(exit_status)
}

fn parse_command(args: FileArgs) -> Result<ExitStatus> {
    let started = Instant::now();
    let source = read_source(&args.file)?;
    let lex_result = lexer::lex(&source);
    let parse_result = parser::parse(&lex_result.tokens);
    let mut diagnostics = lex_result.diagnostics.clone();
    diagnostics.extend(parse_result.diagnostics.clone());
    let file_display = args.file.display().to_string();

    let mut report = new_report("parse", &file_display, "ast");
    report.section("Diagnostics", |section| {
        render_diagnostic_summary(section, &diagnostics);
    });
    report.blank();
    report.section("Output", |section| {
        match parse_result.program {
            Some(program) => {
                section.section("AST", |ast| {
                    render_text_lines(ast, format!("{program:#?}"));
                });
            }
            None => {
                section.line("parse failed: no program produced");
            }
        }
    });
    report.blank();
    report.section("Next", |section| {
        section.command("inspect", command_inspect_ast(&args.file));
        section.command("check", command_check(&args.file));
    });

    let exit_status = if has_errors(&diagnostics) {
        ExitStatus::Failure
    } else {
        ExitStatus::Success
    };
    print_output(report.finish(footer_status(exit_status), started.elapsed()));
    Ok(exit_status)
}

fn check_command(args: FileArgs) -> Result<ExitStatus> {
    let started = Instant::now();
    let source = read_source(&args.file)?;
    let check_result = pcf::check(&source)?;
    let file_display = args.file.display().to_string();

    let mut report = new_report("check", &file_display, "diagnostics");
    report.section("Diagnostics", |section| {
        render_diagnostic_summary(section, &check_result.diagnostics);
        if !check_result.diagnostics.is_empty() {
            section.blank();
            render_diagnostic_output(section, &check_result.diagnostics, &args.file, &source);
        }
    });
    report.blank();
    report.section("Content", |section| render_check_content(section, &check_result.outputs));
    report.blank();
    report.section("Next", |section| {
        section.command("inspect", command_inspect_ast(&args.file));
        section.command("run", command_run(&args.file));
    });

    let exit_status = if has_errors(&check_result.diagnostics) {
        ExitStatus::Failure
    } else {
        ExitStatus::Success
    };
    print_output(report.finish(footer_status(exit_status), started.elapsed()));
    Ok(exit_status)
}

fn run_command(args: FileArgs) -> Result<ExitStatus> {
    let started = Instant::now();
    let source = read_source(&args.file)?;
    let file_display = args.file.display().to_string();
    let mut report = new_report("run", &file_display, "output");

    let (output_lines, exit_status) = match pcf::execute(&source) {
        Ok(value) => (vec![format!("{value:#?}")], ExitStatus::Success),
        Err(error) => (vec![format!("execution failed: {}", error.message)], ExitStatus::Failure),
    };

    report.section("Diagnostics", |section| render_diagnostic_summary(section, &[]));
    report.blank();
    report.section("Output", |section| render_text_lines(section, output_lines.join("\n")));
    report.blank();
    report.section("Next", |section| {
        section.command("inspect", command_inspect_tokens(&args.file));
        section.command("check", command_check(&args.file));
    });

    print_output(report.finish(footer_status(exit_status), started.elapsed()));
    Ok(exit_status)
}

fn read_source(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

fn new_report(profile: &str, target: &str, state: &str) -> ui::TerminalReport {
    let mut report = ui::TerminalReport::new("PCF", env!("CARGO_PKG_VERSION"));
    report.field("Profile", profile);
    report.field("Target", target);
    report.field("Module", target);
    report.field("State", state);
    report.blank();
    report
}

fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| matches!(diagnostic.severity, pcf::diagnostics::Severity::Error))
}

fn render_diagnostic_summary(section: &mut ui::SectionBuilder, diagnostics: &[Diagnostic]) {
    let errors = diagnostics.iter().filter(|d| matches!(d.severity, pcf::diagnostics::Severity::Error)).count();
    let warnings = diagnostics.iter().filter(|d| matches!(d.severity, pcf::diagnostics::Severity::Warning)).count();
    section.field("Errors", errors.to_string());
    section.field("Warnings", warnings.to_string());
    section.field("Status", ui::diagnostic_status(errors, warnings));
}

fn render_diagnostic_output(section: &mut ui::SectionBuilder, diagnostics: &[Diagnostic], file: &Path, source: &str) {
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        section.section(ui::format_diagnostic_title(diagnostic), |entry| {
            ui::render_diagnostic_labels(entry, diagnostic, file, source);
        });
        if index + 1 != diagnostics.len() {
            section.blank();
        }
    }
}

fn render_check_content(section: &mut ui::SectionBuilder, outputs: &[pcf::CheckOutput]) {
    section.section("Output", |output| {
        if outputs.is_empty() {
            output.line("No output");
            return;
        }
        for item in outputs {
            output.line(&item.content);
        }
    });
}

fn render_tokens<T: std::fmt::Debug>(section: &mut ui::SectionBuilder, tokens: &[T]) {
    section.section("Tokens", |tokens_section| {
        for token in tokens {
            tokens_section.line(format!("{token:?}"));
        }
    });
}

fn render_text_lines(section: &mut ui::SectionBuilder, text: String) {
    for line in text.lines() {
        section.line(line);
    }
}

fn command_parse(path: &Path) -> String { format!("pcf parse {}", path.display()) }
fn command_check(path: &Path) -> String { format!("pcf check {}", path.display()) }
fn command_run(path: &Path) -> String { format!("pcf run {}", path.display()) }
fn command_inspect_tokens(path: &Path) -> String { format!("pcf inspect {} --tokens", path.display()) }
fn command_inspect_ast(path: &Path) -> String { format!("pcf inspect {} --ast", path.display()) }

fn inspect_state(tokens_selected: bool, ast_selected: bool) -> &'static str {
    match (tokens_selected, ast_selected) {
        (true, true) => "tokens + ast",
        (true, false) => "tokens",
        (false, true) => "ast",
        (false, false) => "none",
    }
}

fn footer_status(status: ExitStatus) -> FooterStatus {
    match status {
        ExitStatus::Success => FooterStatus::Completed,
        ExitStatus::Failure => FooterStatus::Failed,
    }
}

fn print_output(text: String) { println!("{text}"); }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_errors_from_parse_diagnostics() {
        let source = lexer::lex("unknown\n");
        let parse_result = parser::parse(&source.tokens);
        assert!(!source.diagnostics.iter().chain(parse_result.diagnostics.iter()).collect::<Vec<_>>().is_empty());
        assert!(has_errors(&parse_result.diagnostics));
    }

    #[test]
    fn renders_static_output_content() {
        let mut report = ui::TerminalReport::new("PCF", "0.1.0");
        report.section("Content", |section| {
            render_check_content(section, &[
                pcf::CheckOutput { content: "hello".to_string(), span: pcf_span::Span { start: 0, end: 5 } },
                pcf::CheckOutput { content: "world".to_string(), span: pcf_span::Span { start: 6, end: 11 } },
            ]);
        });
        let rendered = report.finish(FooterStatus::Completed, std::time::Duration::from_millis(1));
        assert!(rendered.contains("├─ Content"));
        assert!(rendered.contains("│  ╰─ Output"));
        assert!(rendered.contains("   ├─ hello"));
        assert!(rendered.contains("   ╰─ world"));
    }
}
