use std::collections::HashMap;

use crate::{ScopeId, SymbolId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub symbols: HashMap<String, SymbolId>,
}
