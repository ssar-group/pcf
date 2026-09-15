use pcf_ast::Program;

use crate::{RuntimeContext, RuntimeError, Value};

#[derive(Debug, Default)]
pub struct Evaluator;

impl Evaluator {
    pub fn evaluate_program(
        &mut self,
        _program: &Program,
        _context: &mut RuntimeContext<'_>,
    ) -> Result<Value, RuntimeError> {
        Ok(Value::Null)
    }
}
