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
                Item::AllowMethods(_) | Item::BlockMethods(_) | Item::Process(_) => {}
                Item::Export(export) => {
                    if let Item::Function(function) = export.declaration.as_ref() {
                        let current = scopes.last_mut().expect("scope exists");
                        current.insert(function.name.name.clone(), function.name.span.start);
                    }
                }
            }
        }

        let mut import_paths = HashMap::<String, usize>::new();
        let mut used_names = std::collections::HashSet::<String>::new();
        for item in &program.items {
            match item {
                Item::Import(import) => {
                    let path = import
                        .path
                        .iter()
                        .map(|part| part.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::");
                    if let Some(previous) = import_paths.insert(path.clone(), import.span.start) {
                        diagnostics.push(semantic_diagnostic(
                            "PCF3004",
                            Severity::Warning,
                            "duplicate import",
                            import.span,
                            Some(previous),
                        ));
                    }
                }
                Item::Variable(variable) => {
                    if let Some(expr) = &variable.statement.value {
                        collect_expression_names(expr, &mut used_names);
                    }
                }
                Item::Function(function) => {
                    for statement in &function.body {
                        collect_statement_names(statement, &mut used_names);
                    }
                }
                Item::Statement(statement) => collect_statement_names(statement, &mut used_names),
                Item::Process(route) => {
                    for statement in &route.body {
                        collect_statement_names(statement, &mut used_names);
                    }
                }
                Item::Export(export) => {
                    if let Item::Function(function) = export.declaration.as_ref() {
                        for statement in &function.body {
                            collect_statement_names(statement, &mut used_names);
                        }
                    }
                }
                _ => {}
            }
        }
        for item in &program.items {
            match item {
                Item::Import(import) => {
                    if let Some(alias) = import.path.last()
                        && !used_names.contains(&alias.name)
                    {
                        diagnostics.push(semantic_diagnostic(
                            "PCF3005",
                            Severity::Warning,
                            "unused import",
                            alias.span,
                            None,
                        ));
                    }
                }
                Item::Variable(variable) => {
                    let declaration = &variable.statement;
                    if !used_names.contains(&declaration.name.name) {
                        diagnostics.push(semantic_diagnostic(
                            "PCF3006",
                            Severity::Warning,
                            "unused variable",
                            declaration.name.span,
                            None,
                        ));
                    }
                }
                Item::Function(function) => {
                    analyze_unreachable(&function.body, &mut diagnostics);
                    let mut function_uses = std::collections::HashSet::new();
                    for statement in &function.body {
                        collect_statement_names(statement, &mut function_uses);
                    }
                    for parameter in &function.parameters {
                        if !function_uses.contains(&parameter.name.name) {
                            diagnostics.push(semantic_diagnostic(
                                "PCF3006",
                                Severity::Warning,
                                "unused parameter",
                                parameter.span,
                                None,
                            ));
                        }
                    }
                    collect_unused_locals(&function.body, &function_uses, &mut diagnostics);
                }
                Item::Statement(statement) => {
                    analyze_unreachable(std::slice::from_ref(statement), &mut diagnostics)
                }
                Item::Process(route) => {
                    analyze_unreachable(&route.body, &mut diagnostics);
                    let mut route_uses = std::collections::HashSet::new();
                    for statement in &route.body {
                        collect_statement_names(statement, &mut route_uses);
                    }
                    collect_unused_locals(&route.body, &route_uses, &mut diagnostics);
                }
                _ => {}
            }
        }

        AnalysisResult { diagnostics }
    }
}

fn collect_expression_names(
    expression: &pcf_ast::Expression,
    names: &mut std::collections::HashSet<String>,
) {
    use pcf_ast::Expression;
    match expression {
        Expression::Identifier(id) => {
            names.insert(id.name.clone());
        }
        Expression::Unary(u) => collect_expression_names(&u.operand, names),
        Expression::Binary(b) => {
            collect_expression_names(&b.left, names);
            collect_expression_names(&b.right, names);
        }
        Expression::Group(inner) => collect_expression_names(inner, names),
        Expression::Call(call) => {
            collect_expression_names(&call.callee, names);
            for arg in &call.arguments {
                collect_expression_names(arg, names);
            }
        }
        Expression::Member(member) => {
            collect_expression_names(&member.object, names);
        }
        Expression::Literal(_) => {}
    }
}

fn collect_statement_names(statement: &Statement, names: &mut std::collections::HashSet<String>) {
    match statement {
        Statement::Variable(v) => {
            if let Some(expr) = &v.value {
                collect_expression_names(expr, names);
            }
        }
        Statement::Expression(e) => collect_expression_names(&e.expression, names),
        Statement::Return(r) => {
            if let Some(expr) = &r.value {
                collect_expression_names(expr, names);
            }
        }
        Statement::Block(b) => {
            for s in &b.statements {
                collect_statement_names(s, names);
            }
        }
        Statement::If(f) => {
            collect_expression_names(&f.condition, names);
            for s in &f.then_branch.statements {
                collect_statement_names(s, names);
            }
            if let Some(b) = &f.else_branch {
                for s in &b.statements {
                    collect_statement_names(s, names);
                }
            }
        }
        Statement::While(f) => {
            collect_expression_names(&f.condition, names);
            for s in &f.body.statements {
                collect_statement_names(s, names);
            }
        }
        Statement::For(f) => {
            collect_expression_names(&f.iterable, names);
            for s in &f.body.statements {
                collect_statement_names(s, names);
            }
        }
        Statement::Output(_) => {}
    }
}

fn analyze_unreachable(statements: &[Statement], diagnostics: &mut Vec<Diagnostic>) {
    let mut returned = false;
    for statement in statements {
        if returned {
            diagnostics.push(semantic_diagnostic(
                "PCF3007",
                Severity::Warning,
                "unreachable statement",
                statement_span(statement),
                None,
            ));
        }
        if matches!(statement, Statement::Return(_)) {
            returned = true;
        }
        match statement {
            Statement::Block(b) => analyze_unreachable(&b.statements, diagnostics),
            Statement::If(f) => {
                analyze_unreachable(&f.then_branch.statements, diagnostics);
                if let Some(b) = &f.else_branch {
                    analyze_unreachable(&b.statements, diagnostics);
                }
            }
            Statement::While(f) => analyze_unreachable(&f.body.statements, diagnostics),
            Statement::For(f) => analyze_unreachable(&f.body.statements, diagnostics),
            _ => {}
        }
    }
}

fn collect_unused_locals(
    statements: &[Statement],
    used_names: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for statement in statements {
        match statement {
            Statement::Variable(variable) if !used_names.contains(&variable.name.name) => {
                diagnostics.push(semantic_diagnostic(
                    "PCF3006",
                    Severity::Warning,
                    "unused variable",
                    variable.name.span,
                    None,
                ));
            }
            Statement::Block(block) => {
                collect_unused_locals(&block.statements, used_names, diagnostics)
            }
            Statement::If(flow) => {
                collect_unused_locals(&flow.then_branch.statements, used_names, diagnostics);
                if let Some(block) = &flow.else_branch {
                    collect_unused_locals(&block.statements, used_names, diagnostics);
                }
            }
            Statement::While(flow) => {
                collect_unused_locals(&flow.body.statements, used_names, diagnostics)
            }
            Statement::For(flow) => {
                if !used_names.contains(&flow.item.name) {
                    diagnostics.push(semantic_diagnostic(
                        "PCF3006",
                        Severity::Warning,
                        "unused loop binding",
                        flow.item.span,
                        None,
                    ));
                }
                collect_unused_locals(&flow.body.statements, used_names, diagnostics);
            }
            _ => {}
        }
    }
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::Variable(s) => s.span,
        Statement::Output(s) => s.span,
        Statement::Expression(s) => s.span,
        Statement::Block(s) => s.span,
        Statement::Return(s) => s.span,
        Statement::If(s) => s.span,
        Statement::While(s) => s.span,
        Statement::For(s) => s.span,
    }
}

fn semantic_diagnostic(
    code: &'static str,
    severity: Severity,
    message: &str,
    span: Span,
    previous: Option<usize>,
) -> Diagnostic {
    let mut labels = vec![Label {
        span,
        message: None,
        primary: true,
    }];
    if let Some(start) = previous {
        labels.push(Label {
            span: Span { start, end: start },
            message: Some("previous declaration".into()),
            primary: false,
        });
    }
    Diagnostic {
        severity,
        code: DiagnosticCode(code),
        message: message.to_string(),
        labels,
        notes: Vec::new(),
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
        Statement::If(flow) => {
            scopes.push(HashMap::new());
            analyze_statements(&flow.then_branch.statements, scopes, diagnostics);
            scopes.pop();
            if let Some(block) = &flow.else_branch {
                scopes.push(HashMap::new());
                analyze_statements(&block.statements, scopes, diagnostics);
                scopes.pop();
            }
        }
        Statement::While(flow) => {
            scopes.push(HashMap::new());
            analyze_statements(&flow.body.statements, scopes, diagnostics);
            scopes.pop();
        }
        Statement::For(flow) => {
            scopes.push(HashMap::new());
            scopes
                .last_mut()
                .expect("scope exists")
                .insert(flow.item.name.clone(), flow.item.span.start);
            analyze_statements(&flow.body.statements, scopes, diagnostics);
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
