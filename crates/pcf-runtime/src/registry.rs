use std::collections::BTreeMap;

use crate::{NativeFunctionId, Value};

#[derive(Debug, Clone, Default)]
pub struct RuntimeRegistry {
    pub modules: BTreeMap<String, Value>,
    pub native_functions: BTreeMap<String, NativeFunctionId>,
}
