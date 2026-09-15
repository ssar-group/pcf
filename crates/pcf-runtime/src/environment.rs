use std::collections::BTreeMap;

use crate::Value;

#[derive(Debug, Clone, Default)]
pub struct RuntimeScope {
    pub values: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub scopes: Vec<RuntimeScope>,
}
