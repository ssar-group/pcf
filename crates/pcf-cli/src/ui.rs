use std::{fmt::Write as _, path::Path, time::Duration};

use pcf::diagnostics::{Diagnostic, Label, Severity};

const ROOT_MIN_FIELD_WIDTH: usize = 12;
const SECTION_MIN_FIELD_WIDTH: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterStatus {
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TerminalReport {
    project_name: String,
    version: String,
    items: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct SectionBuilder {
    items: Vec<Node>,
}

#[derive(Debug, Clone)]
enum Node {
    Field { label: String, value: String },
    Line(String),
    Blank,
    Section { title: String, items: Vec<Node> },
}

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
        let _ = writeln!(
            rendered,
            "╭─[{}] {} v{}",
            self.project_name, self.project_name, self.version
        );
        let _ = writeln!(rendered, "│");
        render_nodes(&self.items, "", &mut rendered, ROOT_MIN_FIELD_WIDTH);
        let footer_label = match footer {
            FooterStatus::Completed => "Completed",
            FooterStatus::Failed => "Failed",
        };
        let _ = writeln!(
            rendered,
            "╰─ {} in {}",
            footer_label,
            format_duration(duration)
        );
        rendered
    }
}

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
        "{}[{}] {}",
        severity_name(diagnostic.severity),
        diagnostic.code.0,
        diagnostic.message
    )
}

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
        section.line(format_label(file, source, label));
    }

    for note in &diagnostic.notes {
        section.line(format!("note: {}", note));
    }
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

fn render_nodes(nodes: &[Node], prefix: &str, rendered: &mut String, min_field_width: usize) {
    let field_width = field_width(nodes, min_field_width);

    for (index, node) in nodes.iter().enumerate() {
        let is_last = index + 1 == nodes.len();
        match node {
            Node::Field { label, value } => {
                let branch = if is_last { "╰─" } else { "├─" };
                let _ = writeln!(
                    rendered,
                    "{prefix}{branch} {label:<width$} {value}",
                    width = field_width
                );
            }
            Node::Line(text) => {
                let branch = if is_last { "╰─" } else { "├─" };
                let _ = writeln!(rendered, "{prefix}{branch} {text}");
            }
            Node::Blank => {
                let _ = writeln!(rendered, "{prefix}│");
            }
            Node::Section { title, items } => {
                let branch = if is_last { "╰─" } else { "├─" };
                let _ = writeln!(rendered, "{prefix}{branch} {title}");
                // Keep the vertical continuation bar for child items so hierarchy
                // remains visible even when the parent is the last sibling.
                let next_prefix = format!("{prefix}│  ");
                render_nodes(items, &next_prefix, rendered, SECTION_MIN_FIELD_WIDTH);
            }
        }
    }
}

fn field_width(nodes: &[Node], min_field_width: usize) -> usize {
    nodes
        .iter()
        .filter_map(|node| match node {
            Node::Field { label, .. } => Some(label.chars().count()),
            _ => None,
        })
        .max()
        .map(|width| width.max(min_field_width))
        .unwrap_or(min_field_width)
}

fn format_label(file: &Path, source: &str, label: &Label) -> String {
    let (line, column) = location_for_offset(source, label.span.start);
    match &label.message {
        Some(message) if !message.is_empty() => {
            format!("{}:{}:{} - {}", file.display(), line, column, message)
        }
        _ => format!("{}:{}:{}", file.display(), line, column),
    }
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
            column += 1;
        }

        byte_index += ch.len_utf8();
    }

    (line, column)
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
        report.blank();
        report.section("Next", |section| {
            section.command("inspect", "pcf inspect ./example/test/com.pcf --ast");
            section.command("run", "pcf run ./example/test/com.pcf");
        });

        let rendered = report.finish(FooterStatus::Completed, Duration::from_millis(1));
        // Exact rendering assertion to prevent regressions in output formatting.
        assert_eq!(
            rendered,
            "╭─[PCF] PCF v0.1.0\n│\n├─ Profile      check\n├─ Target       ./example/test/com.pcf\n├─ Module       ./example/test/com.pcf\n├─ State        diagnostics\n│\n├─ Diagnostics\n│  ├─ Errors    0\n│  ├─ Warnings  0\n│  ╰─ Status    clean\n│\n├─ Output\n│  ╰─ No diagnostics found\n│\n╰─ Next\n│  ├─ inspect   pcf inspect ./example/test/com.pcf --ast\n│  ╰─ run       pcf run ./example/test/com.pcf\n╰─ Completed in 1ms\n",
        );
    }

    #[test]
    fn renders_nested_diagnostics() {
        let diagnostics = [
            diagnostic(
                Severity::Error,
                "PCF0001",
                "expected `;`",
                Span { start: 12, end: 13 },
            ),
            Diagnostic {
                severity: Severity::Warning,
                code: DiagnosticCode("PCF0002"),
                message: "unused value".to_string(),
                labels: vec![Label {
                    span: Span { start: 24, end: 25 },
                    message: None,
                    primary: true,
                }],
                notes: Vec::new(),
            },
        ];

        let mut report = TerminalReport::new("PCF", "0.1.0");
        report.section("Diagnostics", |section| {
            section.field("Errors", "1");
            section.field("Warnings", "1");
            section.field("Status", "failed");
        });
        report.blank();
        report.section("Output", |section| {
            for (index, diagnostic) in diagnostics.iter().enumerate() {
                section.section(format_diagnostic_title(diagnostic), |entry| {
                    render_diagnostic_labels(
                        entry,
                        diagnostic,
                        Path::new("./example/test/com.pcf"),
                        "let value = 1;\nfoo\n",
                    );
                });

                if index + 1 != diagnostics.len() {
                    section.blank();
                }
            }
        });

        let rendered = report.finish(FooterStatus::Failed, Duration::from_millis(3));

        assert!(rendered.contains("├─ Diagnostics"));
        assert!(rendered.contains("╰─ Status"));
        assert!(rendered.contains("failed"));
        assert!(rendered.contains("╰─ Output"));
        assert!(rendered.contains("├─ error[PCF0001] expected `;`"));
        assert!(rendered.contains("primary label"));
        assert!(rendered.contains("./example/test/com.pcf"));
        assert!(rendered.contains("│"));
        assert!(rendered.contains("╰─ warning[PCF0002] unused value"));
        assert!(rendered.contains("╰─ Failed in 3ms"));
    }

    #[test]
    fn formats_durations() {
        assert_eq!(format_duration(Duration::from_nanos(184)), "184ns");
        assert_eq!(format_duration(Duration::from_micros(184)), "184µs");
        assert_eq!(format_duration(Duration::from_millis(1)), "1ms");
        assert_eq!(format_duration(Duration::from_millis(18)), "18ms");
        assert_eq!(format_duration(Duration::from_millis(1_200)), "1.2s");
    }

    #[test]
    fn aligns_fields_with_shared_width() {
        let mut report = TerminalReport::new("PCF", "0.1.0");
        report.field("Profile", "check");
        report.field("Target", "./example/test/com.pcf");
        report.field("Module", "./example/test/com.pcf");
        report.field("State", "diagnostics");

        let rendered = report.finish(FooterStatus::Completed, Duration::from_millis(1));
        let profile_line = rendered
            .lines()
            .find(|line| line.contains("Profile"))
            .expect("profile line");
        let target_line = rendered
            .lines()
            .find(|line| line.contains("Target"))
            .expect("target line");

        let profile_value_column = profile_line.find("check").expect("profile value");
        let target_value_column = target_line
            .find("./example/test/com.pcf")
            .expect("target value");

        assert_eq!(profile_value_column, target_value_column);
    }
}
