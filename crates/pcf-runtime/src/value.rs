use std::collections::BTreeMap;

use crate::{FunctionValue, ModuleValue, NativeFunctionId};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Function(FunctionValue),
    NativeFunction(NativeFunctionId),
    Module(ModuleValue),
}
