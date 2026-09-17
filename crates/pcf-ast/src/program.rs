use pcf_span::Span;

use crate::{
    FunctionDeclaration, ImportDeclaration, ModuleDeclaration, SchemaDeclaration, Statement,
    VariableDeclaration,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Import(ImportDeclaration),
    Module(ModuleDeclaration),
    Function(FunctionDeclaration),
    Variable(VariableDeclaration),
    Schema(SchemaDeclaration),
    Statement(Statement),
}
