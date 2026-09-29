use std::collections::HashSet;

use pcf_ast::{Expression, Statement};
use pcf_ast::{Item, Program};
use pcf_diagnostics::{Diagnostic, DiagnosticCode, Label, Severity};
use pcf_span::Span;

use crate::{
    ModulePath, ResolveError, ResolvedModule, Scope, ScopeId, Symbol, SymbolId, SymbolKind,
    SymbolTable,
};

#[derive(Debug, Default)]
pub struct Resolver;

impl Resolver {
    /// Resolve lexical references and return structured diagnostics.
    pub fn resolve_diagnostics(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut context = ResolutionContext::default();
        for item in &program.items {
            match item {
                Item::Import(import) => {
                    if let Some(name) = import.path.last() {
                        context.declare(&name.name, name.span, SymbolKind::Import);
                    }
                }
                Item::Module(module) => {
                    context.declare(&module.name.name, module.name.span, SymbolKind::Module);
                }
                Item::Schema(schema) => {
                    context.declare(&schema.name.name, schema.name.span, SymbolKind::Schema);
                }
                Item::Function(function) => {
                    context.declare(
                        &function.name.name,
                        function.name.span,
                        SymbolKind::Function,
                    );
                }
                Item::Variable(variable) => {
                    let kind = if variable.statement.mutable {
                        SymbolKind::Variable
                    } else {
                        SymbolKind::Constant
                    };
                    context.declare(
                        &variable.statement.name.name,
                        variable.statement.name.span,
                        kind,
                    );
                }
                Item::Export(export) => match export.declaration.as_ref() {
                    Item::Function(f) => {
                        context.declare(&f.name.name, f.name.span, SymbolKind::Function);
                    }
                    Item::Variable(v) => {
                        let kind = if v.statement.mutable {
                            SymbolKind::Variable
                        } else {
                            SymbolKind::Constant
                        };
                        context.declare(&v.statement.name.name, v.statement.name.span, kind);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        for item in &program.items {
            match item {
                Item::Function(function) => {
                    context.enter();
                    for p in &function.parameters {
                        context.declare(&p.name.name, p.span, SymbolKind::Parameter);
                    }
                    for s in &function.body {
                        resolve_statement(s, &mut context, &mut diagnostics);
                    }
                    context.leave();
                }
                Item::Variable(variable) => {
                    if let Some(expr) = &variable.statement.value {
                        resolve_expression(expr, &context, &mut diagnostics);
                    }
                }
                Item::Process(process) => {
                    context.enter();
                    for s in &process.body {
                        resolve_statement(s, &mut context, &mut diagnostics);
                    }
                    context.leave();
                }
                Item::Statement(statement) => {
                    resolve_statement(statement, &mut context, &mut diagnostics)
                }
                Item::Export(export) => {
                    if let Item::Function(function) = export.declaration.as_ref() {
                        context.enter();
                        for p in &function.parameters {
                            context.declare(&p.name.name, p.span, SymbolKind::Parameter);
                        }
                        for s in &function.body {
                            resolve_statement(s, &mut context, &mut diagnostics);
                        }
                        context.leave();
                    } else if let Item::Variable(variable) = export.declaration.as_ref()
                        && let Some(expression) = &variable.statement.value
                    {
                        resolve_expression(expression, &context, &mut diagnostics);
                    }
                }
                _ => {}
            }
        }
        diagnostics
    }
    /// Resolve the program into a concrete module path. The previous API
    /// returned an Option inside ResolveResult even though errors were already
    /// represented by Result. Simplify by returning ResolvedModule directly on
    /// success.
    pub fn resolve_program(&self, program: &Program) -> Result<ResolvedModule, ResolveError> {
        let mut segments = HashSet::new();
        let mut modules = Vec::new();

        for item in &program.items {
            match item {
                Item::Module(module) => {
                    modules.push(module.name.name.clone());
                    segments.insert(module.name.name.clone());
                }
                Item::Import(import) => {
                    if import.path.is_empty() {
                        return Err(ResolveError {
                            message: "import declarations require a path".to_string(),
                        });
                    }
                    for identifier in &import.path {
                        segments.insert(identifier.name.clone());
                    }
                    modules.extend(import.path.iter().map(|identifier| identifier.name.clone()));
                }
                Item::Function(function) => {
                    segments.insert(function.name.name.clone());
                }
                Item::Variable(variable) => {
                    segments.insert(variable.statement.name.name.clone());
                }
                Item::Schema(schema) => {
                    segments.insert(schema.name.name.clone());
                }
                Item::Statement(_) => {}
                Item::AllowMethods(_) | Item::BlockMethods(_) => {}
                Item::Process(_) => {}
                Item::Export(export) => {
                    if let Item::Function(function) = export.declaration.as_ref() {
                        segments.insert(function.name.name.clone());
                    }
                    if let Item::Variable(variable) = export.declaration.as_ref() {
                        segments.insert(variable.statement.name.name.clone());
                    }
                }
            }
        }

        let mut ordered = modules;
        ordered.sort();
        ordered.dedup();

        Ok(ResolvedModule {
            path: ModulePath { segments: ordered },
        })
    }
}

#[derive(Debug)]
struct ResolutionContext {
    scopes: Vec<Scope>,
    current: ScopeId,
    symbols: SymbolTable,
}

impl Default for ResolutionContext {
    fn default() -> Self {
        Self {
            scopes: vec![Scope {
                id: ScopeId(0),
                parent: None,
                symbols: Default::default(),
            }],
            current: ScopeId(0),
            symbols: SymbolTable::default(),
        }
    }
}

impl ResolutionContext {
    fn enter(&mut self) {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope {
            id,
            parent: Some(self.current),
            symbols: Default::default(),
        });
        self.current = id;
    }

    fn leave(&mut self) {
        if let Some(parent) = self
            .scopes
            .get(self.current.0)
            .and_then(|scope| scope.parent)
        {
            self.current = parent;
        }
    }

    fn declare(&mut self, name: &str, span: Span, kind: SymbolKind) {
        let id = SymbolId(self.symbols.symbols.len());
        self.symbols.symbols.push(Symbol {
            id,
            name: name.to_string(),
            kind,
            scope: self.current,
            declaration_span: span,
        });
        if let Some(scope) = self.scopes.get_mut(self.current.0) {
            scope.symbols.insert(name.to_string(), id);
        }
    }

    fn contains(&self, name: &str) -> bool {
        let mut current = Some(self.current);
        while let Some(id) = current {
            let Some(scope) = self.scopes.get(id.0) else {
                return false;
            };
            if scope.symbols.contains_key(name) {
                return true;
            }
            current = scope.parent;
        }
        false
    }
}

fn resolve_statement(
    statement: &Statement,
    context: &mut ResolutionContext,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Statement::Variable(v) => {
            if let Some(e) = &v.value {
                resolve_expression(e, context, diagnostics);
            }
            context.declare(
                &v.name.name,
                v.name.span,
                if v.mutable {
                    SymbolKind::Variable
                } else {
                    SymbolKind::Constant
                },
            );
        }
        Statement::Expression(e) => resolve_expression(&e.expression, context, diagnostics),
        Statement::Return(r) => {
            if let Some(e) = &r.value {
                resolve_expression(e, context, diagnostics);
            }
        }
        Statement::Block(b) => {
            context.enter();
            for s in &b.statements {
                resolve_statement(s, context, diagnostics);
            }
            context.leave();
        }
        Statement::If(f) => {
            resolve_expression(&f.condition, context, diagnostics);
            context.enter();
            for s in &f.then_branch.statements {
                resolve_statement(s, context, diagnostics);
            }
            context.leave();
            if let Some(b) = &f.else_branch {
                context.enter();
                for s in &b.statements {
                    resolve_statement(s, context, diagnostics);
                }
                context.leave();
            }
        }
        Statement::While(f) => {
            resolve_expression(&f.condition, context, diagnostics);
            context.enter();
            for s in &f.body.statements {
                resolve_statement(s, context, diagnostics);
            }
            context.leave();
        }
        Statement::For(f) => {
            resolve_expression(&f.iterable, context, diagnostics);
            context.enter();
            context.declare(&f.item.name, f.item.span, SymbolKind::Variable);
            for s in &f.body.statements {
                resolve_statement(s, context, diagnostics);
            }
            context.leave();
        }
        Statement::Output(_) => {}
    }
}

fn resolve_expression(
    expression: &Expression,
    context: &ResolutionContext,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match expression {
        Expression::Identifier(id) => {
            if !context.contains(&id.name) {
                diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: DiagnosticCode("PCF2003"),
                    message: format!("unknown identifier `{}`", id.name),
                    labels: vec![Label {
                        span: id.span,
                        message: Some("no declaration is visible here".into()),
                        primary: true,
                    }],
                    notes: Vec::new(),
                });
            }
        }
        Expression::Unary(u) => resolve_expression(&u.operand, context, diagnostics),
        Expression::Binary(b) => {
            resolve_expression(&b.left, context, diagnostics);
            resolve_expression(&b.right, context, diagnostics);
        }
        Expression::Group(inner) => resolve_expression(inner, context, diagnostics),
        Expression::Call(call) => {
            resolve_expression(&call.callee, context, diagnostics);
            for arg in &call.arguments {
                resolve_expression(arg, context, diagnostics);
            }
        }
        Expression::Member(member) => resolve_expression(&member.object, context, diagnostics),
        Expression::Literal(_) => {}
    }
}
