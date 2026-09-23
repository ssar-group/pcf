use std::{
    env,
    fmt::Write as _,
    io::{self, IsTerminal},
    path::Path,
    time::Duration,
};

use pcf::diagnostics::{Diagnostic, Label, Severity};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterStatus {
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleKind {
    Error,
    Warning,
    Success,
    Info,
    Hint,
    Command,
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalTheme {
    enabled: bool,
}

impl TerminalTheme {
    pub fn from_mode(mode: ColorMode) -> Self {
        let env_disabled = env::var_os("NO_COLOR").is_some();
        let enabled = match mode {
            ColorMode::Auto => !env_disabled && io::stderr().is_terminal(),
            ColorMode::Always => !env_disabled,
            ColorMode::Never => false,
        };
        Self { enabled }
    }

    #[allow(dead_code)]
    pub fn is_enabled(self) -> bool {
        self.enabled
    }

    pub fn paint(self, kind: StyleKind, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }

        let ansi = match kind {
            StyleKind::Error => "\u{1b}[31;1m",
            StyleKind::Warning => "\u{1b}[33;1m",
            StyleKind::Success => "\u{1b}[32;1m",
            StyleKind::Info => "\u{1b}[36m",
            StyleKind::Hint => "\u{1b}[2m",
            StyleKind::Command => "\u{1b}[1m",
            StyleKind::Normal => "\u{1b}[0m",
        };
        format!("{ansi}{text}\u{1b}[0m")
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TerminalReport {
    project_name: String,
    version: String,
    items: Vec<Node>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SectionBuilder {
    items: Vec<Node>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Node {
    Field { label: String, value: String },
    Line(String),
    Blank,
    Section { title: String, items: Vec<Node> },
}

#[allow(dead_code)]
impl TerminalReport {
    pub fn new(project_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            project_name: project_name.into(),
            version: version.into(),
            items: Vec::new(),
        }
    }

    pub fn field(&mut self, label: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.items.push(Node::Field {
            label: label.into(),
            value: value.into(),
        });
        self
    }

    #[allow(dead_code)]
    pub fn line(&mut self, value: impl Into<String>) -> &mut Self {
        self.items.push(Node::Line(value.into()));
        self
    }

    pub fn blank(&mut self) -> &mut Self {
        self.items.push(Node::Blank);
        self
    }

    pub fn section<F>(&mut self, title: impl Into<String>, build: F) -> &mut Self
    where
        F: FnOnce(&mut SectionBuilder),
    {
        let mut section = SectionBuilder { items: Vec::new() };
        build(&mut section);
        self.items.push(Node::Section {
            title: title.into(),
            items: section.items,
        });
        self
    }

    pub fn finish(self, footer: FooterStatus, duration: Duration) -> String {
        let mut rendered = String::new();
        rendered.push_str(&format!("{} v{}", self.project_name, self.version));
        rendered.push('\n');
        render_nodes(&self.items, &mut rendered, 0, 0);
        if !rendered.ends_with('\n') {
            rendered.push('\n');
        }
        let footer_label = match footer {
            FooterStatus::Completed => "Finished",
            FooterStatus::Failed => "Failed",
        };
        let _ = writeln!(rendered, "{footer_label} in {}", format_duration(duration));
        rendered
    }
}

#[allow(dead_code)]
impl SectionBuilder {
    pub fn field(&mut self, label: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.items.push(Node::Field {
            label: label.into(),
            value: value.into(),
        });
        self
    }

    pub fn command(&mut self, label: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.field(label, value)
    }

    pub fn line(&mut self, value: impl Into<String>) -> &mut Self {
        self.items.push(Node::Line(value.into()));
        self
    }

    pub fn blank(&mut self) -> &mut Self {
        self.items.push(Node::Blank);
        self
    }

    pub fn section<F>(&mut self, title: impl Into<String>, build: F) -> &mut Self
    where
        F: FnOnce(&mut SectionBuilder),
    {
        let mut section = SectionBuilder { items: Vec::new() };
        build(&mut section);
        self.items.push(Node::Section {
            title: title.into(),
            items: section.items,
        });
        self
    }
}

#[allow(dead_code)]
pub fn diagnostic_status(errors: usize, warnings: usize) -> &'static str {
    if errors > 0 {
        "failed"
    } else if warnings > 0 {
        "warning"
    } else {
        "clean"
    }
}

pub fn format_diagnostic_title(diagnostic: &Diagnostic) -> String {
    format!(
        "{}[{}]: {}",
        severity_name(diagnostic.severity),
        diagnostic.code.0,
        diagnostic.message
    )
}

#[allow(dead_code)]
pub fn render_diagnostic_labels(
    section: &mut SectionBuilder,
    diagnostic: &Diagnostic,
    file: &Path,
    source: &str,
) {
    let mut labels: Vec<&Label> = diagnostic
        .labels
        .iter()
        .filter(|label| label.primary)
        .collect();
    labels.extend(diagnostic.labels.iter().filter(|label| !label.primary));

    for label in labels {
        let location = format_label(file, source, label);
        section.line(location);
        if !source.is_empty() {
            if let Some(excerpt) = render_label_excerpt(source, label) {
                for line in excerpt {
                    section.line(line);
                }
            }
        }
    }

    for note in &diagnostic.notes {
        section.line(format!("note: {}", note));
    }
}

pub fn render_diagnostic(diagnostic: &Diagnostic, file: &Path, source: &str) -> Vec<String> {
    let mut output = Vec::new();
    output.push(format_diagnostic_title(diagnostic));

    let mut labels: Vec<&Label> = diagnostic
        .labels
        .iter()
        .filter(|label| label.primary)
        .collect();
    labels.extend(diagnostic.labels.iter().filter(|label| !label.primary));

    for label in labels {
        let location = format_label(file, source, label);
        output.push(location);
        if !source.is_empty() {
            if let Some(excerpt) = render_label_excerpt(source, label) {
                output.extend(excerpt);
            }
        }
    }

    for note in &diagnostic.notes {
        output.push(format!("note: {}", note));
    }

    output
}

pub fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();

    if nanos < 1_000 {
        return format!("{nanos}ns");
    }

    if nanos < 1_000_000 {
        return format!("{}µs", nanos / 1_000);
    }

    if nanos < 1_000_000_000 {
        return format!("{}ms", nanos / 1_000_000);
    }

    let seconds = duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) / 1_000_000_000.0;
    let formatted = format!("{seconds:.1}");
    format!("{}s", formatted.trim_end_matches(".0"))
}

#[allow(dead_code)]
fn render_nodes(nodes: &[Node], rendered: &mut String, indent: usize, depth: usize) {
    for node in nodes {
        match node {
            Node::Field { label, value } => {
                let prefix = " ".repeat(indent + depth * 2);
                let _ = writeln!(rendered, "{prefix}{label}: {value}");
            }
            Node::Line(text) => {
                let prefix = " ".repeat(indent + depth * 2);
                let _ = writeln!(rendered, "{prefix}{text}");
            }
            Node::Blank => {
                let _ = writeln!(rendered);
            }
            Node::Section { title, items } => {
                let prefix = " ".repeat(indent + depth * 2);
                let _ = writeln!(rendered, "{prefix}{title}");
                render_nodes(items, rendered, indent + 2, depth + 1);
            }
        }
    }
}

pub fn format_label(file: &Path, source: &str, label: &Label) -> String {
    let (line, column) = location_for_offset(source, label.span.start);
    format!("--> {}:{}:{}", file.display(), line, column)
}

fn render_label_excerpt(source: &str, label: &Label) -> Option<Vec<String>> {
    let start = label.span.start.min(source.len());
    let end = label.span.end.min(source.len());
    let Some((line_no, line_text)) = line_for_offset(source, start) else {
        return None;
    };
    let start_column = display_column_for_offset(source, start);
    let span_width = display_width_of_span(source, start, end).max(1);
    let marker = if label.primary { '^' } else { '~' };
    let caret = marker.to_string().repeat(span_width);
    let padding = " ".repeat(start_column.saturating_sub(1));
    let line_prefix = format!("{line_no} | ");
    let message = label
        .message
        .as_deref()
        .filter(|message| !message.trim().is_empty())
        .map(|message| format!(" {message}"))
        .unwrap_or_default();

    Some(vec![
        format!("{line_prefix}{line_text}"),
        format!("  | {padding}{caret}{message}"),
    ])
}

fn line_for_offset(source: &str, offset: usize) -> Option<(usize, &str)> {
    let mut current = 0usize;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        let bound = current + line.len();
        if offset < bound {
            return Some((index + 1, line.trim_end_matches('\n')));
        }
        current = bound;
    }

    if source.is_empty() {
        return Some((1, ""));
    }

    let last = source.lines().last()?;
    Some((source.lines().count(), last))
}

fn location_for_offset(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    let mut byte_index = 0usize;

    for ch in source.chars() {
        if byte_index >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += unicode_width(ch);
        }

        byte_index += ch.len_utf8();
    }

    (line, column)
}

fn display_column_for_offset(source: &str, offset: usize) -> usize {
    location_for_offset(source, offset).1
}

fn display_width_of_span(source: &str, start: usize, end: usize) -> usize {
    let start = start.min(source.len());
    let end = end.min(source.len());
    let mut width = 0usize;
    for (index, ch) in source.char_indices() {
        if index < start {
            continue;
        }
        if index >= end {
            break;
        }
        width += unicode_width(ch);
    }
    width.max(1)
}

fn unicode_width(ch: char) -> usize {
    match ch {
        '\t' => 8,
        '\n' => 0,
        _ if ch.is_ascii() => 1,
        _ => 2,
    }
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Hint => "hint",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcf::diagnostics::{Diagnostic, DiagnosticCode, Label, Severity};
    use pcf_span::Span;

    fn diagnostic(severity: Severity, code: &'static str, message: &str, span: Span) -> Diagnostic {
        Diagnostic {
            severity,
            code: DiagnosticCode(code),
            message: message.to_string(),
            labels: vec![Label {
                span,
                message: Some("primary label".to_string()),
                primary: true,
            }],
            notes: vec!["use a different value".to_string()],
        }
    }

    #[test]
    fn renders_clean_report() {
        let mut report = TerminalReport::new("PCF", "0.1.0");
        report.field("Profile", "check");
        report.field("Target", "./example/test/com.pcf");
        report.field("Module", "./example/test/com.pcf");
        report.field("State", "diagnostics");
        report.blank();
        report.section("Diagnostics", |section| {
            section.field("Errors", "0");
            section.field("Warnings", "0");
            section.field("Status", "clean");
        });
        report.blank();
        report.section("Output", |section| {
            section.line("No diagnostics found");
        });

        let rendered = report.finish(FooterStatus::Completed, Duration::from_millis(1));
        assert!(rendered.contains("PCF v0.1.0"));
        assert!(rendered.contains("Profile: check"));
        assert!(rendered.contains("Status: clean"));
        assert!(rendered.contains("Finished in 1ms"));
    }

    #[test]
    fn renders_diagnostic_excerpt() {
        let diagnostic = diagnostic(
            Severity::Error,
            "PCF0001",
            "expected `;`",
            Span { start: 12, end: 13 },
        );

        let rendered = render_diagnostic(
            &diagnostic,
            Path::new("./example/test/com.pcf"),
            "let value = 42\n",
        );
        let text = rendered.join("\n");
        assert!(text.contains("error[PCF0001]: expected `;`"));
        assert!(text.contains("--> ./example/test/com.pcf:"));
        assert!(text.contains("1 | let value = 42"));
        assert!(text.contains("note: use a different value"));
    }

    #[test]
    fn respects_utf8_display_columns() {
        let source = "🙂a\n";
        assert_eq!(location_for_offset(source, 0), (1, 1));
        assert_eq!(location_for_offset(source, 4), (1, 3));
        assert_eq!(location_for_offset(source, 6), (2, 1));
    }
}
