use pcf_span::Span;

use crate::{
    FunctionDeclaration, ImportDeclaration, MethodDeclaration, ModuleDeclaration,
    ProcessDeclaration, SchemaDeclaration, Statement, VariableDeclaration,
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
    AllowMethods(MethodDeclaration),
    BlockMethods(MethodDeclaration),
    Process(ProcessDeclaration),
    Export(ExportDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportDeclaration {
    pub declaration: Box<Item>,
    pub span: Span,
}
