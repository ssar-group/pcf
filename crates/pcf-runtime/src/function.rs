use pcf_ast::FunctionDeclaration;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionValue {
    pub declaration: FunctionDeclaration,
}
