use crate::Symbol;

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}
