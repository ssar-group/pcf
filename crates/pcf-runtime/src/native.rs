use crate::{RuntimeContext, RuntimeError, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NativeFunctionId(pub usize);

pub trait NativeFunction: Send + Sync {
    fn call(
        &self,
        _context: &mut RuntimeContext<'_>,
        _arguments: &[Value],
    ) -> Result<Value, RuntimeError>;
}
