use crate::{
    FunctionDeclaration, ImportDeclaration, ModuleDeclaration, SchemaDeclaration, VariableStatement,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Import(ImportDeclaration),
    Module(ModuleDeclaration),
    Function(FunctionDeclaration),
    Variable(VariableDeclaration),
    Schema(SchemaDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    pub statement: VariableStatement,
}
