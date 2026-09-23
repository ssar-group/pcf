mod ui;

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use pcf::{self, diagnostics::Diagnostic, lexer, parser};

use crate::ui::{ColorMode, FooterStatus, StyleKind, TerminalTheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ColorArg {
    Auto,
    Always,
    Never,
}

impl ColorArg {
    fn into_color_mode(self) -> ColorMode {
        match self {
            Self::Auto => ColorMode::Auto,
            Self::Always => ColorMode::Always,
            Self::Never => ColorMode::Never,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    Success,
    Failure,
}

impl ExitStatus {
    pub fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "pcf", version, about = "PCF command-line interface")]
struct Cli {
    #[arg(long, value_enum, default_value_t = ColorArg::Auto)]
    color: ColorArg,

    #[command(subcommand)]
    command: Commands,
}

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

    #[arg(long)]
    tokens: bool,

    #[arg(long)]
    ast: bool,
}

#[derive(Debug, Parser)]
struct FileArgs {
    file: PathBuf,
}

pub fn run() -> Result<ExitStatus> {
    let cli = Cli::parse();
    let theme = TerminalTheme::from_mode(cli.color.into_color_mode());

    match cli.command {
        Commands::Inspect(args) => inspect(args, theme),
        Commands::Parse(args) => parse_command(args, theme),
        Commands::Check(args) => check_command(args, theme),
        Commands::Run(args) => run_command(args, theme),
    }
}

fn inspect(args: InspectArgs, theme: TerminalTheme) -> Result<ExitStatus> {
    let started = Instant::now();
    let source = read_source(&args.file)?;
    let lex_result = lexer::lex(&source);
    let parse_result = parser::parse(&lex_result.tokens);
    let tokens_selected = args.tokens || !args.ast;
    let ast_selected = args.ast;
    let file_display = args.file.display().to_string();

    let mut diagnostics = lex_result.diagnostics.clone();
    diagnostics.extend(parse_result.diagnostics.clone());
    let exit_status = if has_errors(&diagnostics) {
        ExitStatus::Failure
    } else {
        ExitStatus::Success
    };

    if !diagnostics.is_empty() {
        emit_stderr_lines(render_diagnostics(&diagnostics, &args.file, &source));
    }

    let summary = if exit_status == ExitStatus::Success {
        format!(
            "{} parsed {}",
            theme.paint(StyleKind::Success, "✓"),
            file_display
        )
    } else {
        format!(
            "{} failed to parse {}",
            theme.paint(StyleKind::Error, "✗"),
            file_display
        )
    };
    eprintln!("{summary}");

    if tokens_selected {
        let lines = render_tokens_output(&lex_result.tokens);
        let mut stdout = io::stdout();
        writeln!(stdout, "{lines}")?;
    }

    if ast_selected {
        if tokens_selected {
            println!();
        }
        let program = match parse_result.program {
            Some(program) => format!("{program:#?}"),
            None => "parse failed: no program produced".to_string(),
        };
        println!("AST\n{program}");
    }

    if started.elapsed().as_nanos() > 0 {
        eprintln!("Finished in {}", ui::format_duration(started.elapsed()));
    }

    Ok(exit_status)
}

fn parse_command(args: FileArgs, theme: TerminalTheme) -> Result<ExitStatus> {
    let started = Instant::now();
    let source = read_source(&args.file)?;
    let lex_result = lexer::lex(&source);
    let parse_result = parser::parse(&lex_result.tokens);
    let mut diagnostics = lex_result.diagnostics.clone();
    diagnostics.extend(parse_result.diagnostics.clone());
    let file_display = args.file.display().to_string();

    let exit_status = if has_errors(&diagnostics) {
        ExitStatus::Failure
    } else {
        ExitStatus::Success
    };

    if !diagnostics.is_empty() {
        emit_stderr_lines(render_diagnostics(&diagnostics, &args.file, &source));
    }

    let status = if exit_status == ExitStatus::Success {
        format!(
            "{} parsed {}",
            theme.paint(StyleKind::Success, "✓"),
            file_display
        )
    } else {
        format!(
            "{} failed to parse {}",
            theme.paint(StyleKind::Error, "✗"),
            file_display
        )
    };
    eprintln!("{status}");

    if let Some(program) = parse_result.program {
        println!("AST\n{program:#?}");
    } else {
        println!("parse failed: no program produced");
    }

    eprintln!("Finished in {}", ui::format_duration(started.elapsed()));
    Ok(exit_status)
}

fn check_command(args: FileArgs, theme: TerminalTheme) -> Result<ExitStatus> {
    let started = Instant::now();
    let source = read_source(&args.file)?;
    let check_result = pcf::check(&source)?;
    let file_display = args.file.display().to_string();
    let error_count = check_result
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, pcf::diagnostics::Severity::Error))
        .count();
    let warning_count = check_result
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, pcf::diagnostics::Severity::Warning))
        .count();

    let exit_status = if error_count > 0 {
        ExitStatus::Failure
    } else {
        ExitStatus::Success
    };

    if !check_result.diagnostics.is_empty() {
        emit_stderr_lines(render_diagnostics(
            &check_result.diagnostics,
            &args.file,
            &source,
        ));
        eprintln!();
    }

    if error_count > 0 {
        eprintln!(
            "{} failed {}",
            theme.paint(StyleKind::Error, "✗"),
            file_display
        );
    } else {
        eprintln!(
            "{} checked {}",
            theme.paint(StyleKind::Success, "✓"),
            file_display
        );
    }
    eprintln!(
        "{} errors · {} warnings",
        theme.paint(StyleKind::Normal, &error_count.to_string()),
        theme.paint(StyleKind::Warning, &warning_count.to_string())
    );

    if check_result.outputs.is_empty() {
        eprintln!("no output");
    } else {
        eprintln!("{} outputs", check_result.outputs.len());
        let mut stdout = io::stdout();
        for output in &check_result.outputs {
            writeln!(stdout, "{}", output.content)?;
        }
    }

    eprintln!("Finished in {}", ui::format_duration(started.elapsed()));
    Ok(exit_status)
}

fn run_command(args: FileArgs, theme: TerminalTheme) -> Result<ExitStatus> {
    let started = Instant::now();
    let source = read_source(&args.file)?;
    let file_display = args.file.display().to_string();

    let (output_lines, exit_status) = match pcf::execute(&source) {
        Ok(value) => (vec![format!("{value:#?}")], ExitStatus::Success),
        Err(error) => (
            vec![format!("execution failed: {}", error.message)],
            ExitStatus::Failure,
        ),
    };

    if exit_status == ExitStatus::Failure {
        eprintln!(
            "{} failed {}",
            theme.paint(StyleKind::Error, "✗"),
            file_display
        );
    } else {
        eprintln!(
            "{} ran {}",
            theme.paint(StyleKind::Success, "✓"),
            file_display
        );
    }

    let mut stdout = io::stdout();
    for line in output_lines {
        writeln!(stdout, "{line}")?;
    }
    eprintln!("Finished in {}", ui::format_duration(started.elapsed()));
    Ok(exit_status)
}

fn read_source(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

#[allow(dead_code)]
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
    diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.severity, pcf::diagnostics::Severity::Error))
}

fn render_diagnostics(diagnostics: &[Diagnostic], file: &Path, source: &str) -> Vec<String> {
    let mut output = Vec::new();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        output.extend(ui::render_diagnostic(diagnostic, file, source));
        if index + 1 != diagnostics.len() {
            output.push(String::new());
        }
    }
    output
}

fn emit_stderr_lines(lines: Vec<String>) {
    for line in lines {
        if line.is_empty() {
            eprintln!();
        } else {
            eprintln!("{line}");
        }
    }
}

fn render_tokens_output<T: std::fmt::Debug>(tokens: &[T]) -> String {
    let mut output = String::from("Tokens\n");
    for token in tokens {
        output.push_str(&format!("{token:?}\n"));
    }
    output.trim_end().to_string()
}

#[allow(dead_code)]
fn command_parse(path: &Path) -> String {
    format!("pcf parse {}", path.display())
}

#[allow(dead_code)]
fn command_check(path: &Path) -> String {
    format!("pcf check {}", path.display())
}

#[allow(dead_code)]
fn command_run(path: &Path) -> String {
    format!("pcf run {}", path.display())
}

#[allow(dead_code)]
fn command_inspect_tokens(path: &Path) -> String {
    format!("pcf inspect {} --tokens", path.display())
}

#[allow(dead_code)]
fn command_inspect_ast(path: &Path) -> String {
    format!("pcf inspect {} --ast", path.display())
}

#[allow(dead_code)]
fn inspect_state(tokens_selected: bool, ast_selected: bool) -> &'static str {
    match (tokens_selected, ast_selected) {
        (true, true) => "tokens + ast",
        (true, false) => "tokens",
        (false, true) => "ast",
        (false, false) => "none",
    }
}

#[allow(dead_code)]
fn footer_status(status: ExitStatus) -> FooterStatus {
    match status {
        ExitStatus::Success => FooterStatus::Completed,
        ExitStatus::Failure => FooterStatus::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_errors_from_parse_diagnostics() {
        let source = lexer::lex("unknown\n");
        let parse_result = parser::parse(&source.tokens);
        assert!(!source.diagnostics.is_empty() || !parse_result.diagnostics.is_empty());
        assert!(has_errors(&parse_result.diagnostics));
    }

    #[test]
    fn renders_static_output_content() {
        let mut report = ui::TerminalReport::new("PCF", "0.1.0");
        report.section("Content", |section| {
            section.line("hello");
            section.line("world");
        });
        let rendered = report.finish(FooterStatus::Completed, std::time::Duration::from_millis(1));
        assert!(rendered.contains("Content"));
        assert!(rendered.contains("hello"));
        assert!(rendered.contains("world"));
        assert!(rendered.contains("Finished in 1ms"));
    }
}
