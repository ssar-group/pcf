use pcf_ast::{
    BinaryExpression, BinaryOperator, BlockStatement, Expression, Item, Literal, Program,
    Statement, UnaryExpression, UnaryOperator,
};

use crate::{FunctionValue, RuntimeContext, RuntimeError, Value};

#[derive(Debug, Default)]
pub struct Evaluator;

impl Evaluator {
    pub fn evaluate_program(
        &mut self,
        program: &Program,
        context: &mut RuntimeContext<'_>,
    ) -> Result<Value, RuntimeError> {
        // Start with a fresh top-level scope for program execution
        context.environment.scopes.push(Default::default());
        let mut last = Value::Null;

        for item in &program.items {
            match self.evaluate_item(item, context) {
                Ok(v) => last = v,
                Err(e) => {
                    // If this encodes a return value, extract and return it as a normal result.
                    if let Some(return_value) = extract_return_from_error(&e) {
                        context.environment.scopes.pop();
                        return Ok(return_value);
                    }
                    // ensure we restore the top-level scope before returning
                    context.environment.scopes.pop();
                    return Err(e);
                }
            }
        }

        // Pop the top-level scope
        context.environment.scopes.pop();
        Ok(last)
    }

    fn evaluate_item(
        &mut self,
        item: &Item,
        context: &mut RuntimeContext<'_>,
    ) -> Result<Value, RuntimeError> {
        match item {
            Item::Statement(statement) => self.evaluate_statement(statement, context),
            Item::Function(function) => {
                let value = Value::Function(FunctionValue {
                    declaration: function.clone(),
                });
                set_variable(context, &function.name.name, value);
                Ok(Value::Null)
            }
            Item::Variable(declaration) => {
                let value = match &declaration.statement.value {
                    Some(expression) => self.evaluate_expression(expression, context)?,
                    None => Value::Null,
                };
                set_variable(context, &declaration.statement.name.name, value);
                Ok(Value::Null)
            }
            Item::Import(_) | Item::Module(_) | Item::Schema(_) => Ok(Value::Null),
        }
    }

    fn evaluate_statement(
        &mut self,
        statement: &Statement,
        context: &mut RuntimeContext<'_>,
    ) -> Result<Value, RuntimeError> {
        match statement {
            Statement::Output(output) => {
                let value = self.evaluate_literal(&output.value.value)?;
                Ok(value)
            }
            Statement::Variable(variable) => {
                let value = match &variable.value {
                    Some(expression) => self.evaluate_expression(expression, context)?,
                    None => Value::Null,
                };
                set_variable(context, &variable.name.name, value.clone());
                Ok(value)
            }
            Statement::Expression(expression) => {
                self.evaluate_expression(&expression.expression, context)
            }
            Statement::Block(block) => self.evaluate_block(block, context),
            Statement::Return(return_statement) => {
                // Evaluate the return value and wrap it into a special control
                let value = match &return_statement.value {
                    Some(expression) => self.evaluate_expression(expression, context)?,
                    None => Value::Null,
                };
                // Use a RuntimeError with a special message is not ideal; instead,
                // encode return propagation via a dedicated Result variant by
                // returning an Ok containing a Return value wrapped in Value::Null
                // is insufficient. Implement return propagation via a sentinel
                // by using a RuntimeError with a flag-like message not ideal; but
                // keep internal handling: we'll return a special error type.
                return Err(RuntimeError {
                    message: format!("__PCF_RETURN__:{}", serialize_value_for_return(&value)),
                });
            }
        }
    }

    fn evaluate_block(
        &mut self,
        block: &BlockStatement,
        context: &mut RuntimeContext<'_>,
    ) -> Result<Value, RuntimeError> {
        // Push a nested scope for the block
        context.environment.scopes.push(Default::default());
        let mut last = Value::Null;
        for statement in &block.statements {
            match self.evaluate_statement(statement, context) {
                Ok(v) => last = v,
                Err(e) => {
                    // If this error encodes a return, pop the block scope and re-encode
                    if let Some(return_value) = extract_return_from_error(&e) {
                        context.environment.scopes.pop();
                        return Err(RuntimeError {
                            message: format!(
                                "__PCF_RETURN__:{}",
                                serialize_value_for_return(&return_value)
                            ),
                        });
                    }
                    context.environment.scopes.pop();
                    return Err(e);
                }
            }
        }
        // Pop block scope
        context.environment.scopes.pop();
        Ok(last)
    }

    fn evaluate_expression(
        &mut self,
        expression: &Expression,
        context: &mut RuntimeContext<'_>,
    ) -> Result<Value, RuntimeError> {
        match expression {
            Expression::Literal(literal) => self.evaluate_literal(&literal.value),
            Expression::Identifier(identifier) => resolve_variable(context, &identifier.name),
            Expression::Unary(unary) => self.evaluate_unary(unary, context),
            Expression::Binary(binary) => self.evaluate_binary(binary, context),
            Expression::Group(group) => self.evaluate_expression(group, context),
        }
    }

    fn evaluate_unary(
        &mut self,
        unary: &UnaryExpression,
        context: &mut RuntimeContext<'_>,
    ) -> Result<Value, RuntimeError> {
        let value = self.evaluate_expression(&unary.operand, context)?;
        match unary.operator {
            UnaryOperator::Negate => match value {
                Value::Integer(value) => Ok(Value::Integer(-value)),
                Value::Float(value) => Ok(Value::Float(-value)),
                _ => Err(RuntimeError {
                    message: "unary negation is only valid for numeric values".to_string(),
                }),
            },
            UnaryOperator::Not => match value {
                Value::Boolean(value) => Ok(Value::Boolean(!value)),
                _ => Err(RuntimeError {
                    message: "logical not is only valid for boolean values".to_string(),
                }),
            },
        }
    }

    fn evaluate_binary(
        &mut self,
        binary: &BinaryExpression,
        context: &mut RuntimeContext<'_>,
    ) -> Result<Value, RuntimeError> {
        let left = self.evaluate_expression(&binary.left, context)?;
        let right = self.evaluate_expression(&binary.right, context)?;

        match binary.operator {
            BinaryOperator::Add => apply_add(left, right),
            BinaryOperator::Subtract => apply_subtract(left, right),
            BinaryOperator::Multiply => apply_multiply(left, right),
            BinaryOperator::Divide => apply_divide(left, right),
            BinaryOperator::Modulo => apply_modulo(left, right),
            BinaryOperator::Equal => Ok(Value::Boolean(left == right)),
            BinaryOperator::NotEqual => Ok(Value::Boolean(left != right)),
            BinaryOperator::Less => compare(left, right, |a, b| a < b),
            BinaryOperator::LessEqual => compare(left, right, |a, b| a <= b),
            BinaryOperator::Greater => compare(left, right, |a, b| a > b),
            BinaryOperator::GreaterEqual => compare(left, right, |a, b| a >= b),
            BinaryOperator::And => match (left, right) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Ok(Value::Boolean(lhs && rhs)),
                _ => Err(RuntimeError {
                    message: "logical and requires boolean operands".to_string(),
                }),
            },
            BinaryOperator::Or => match (left, right) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Ok(Value::Boolean(lhs || rhs)),
                _ => Err(RuntimeError {
                    message: "logical or requires boolean operands".to_string(),
                }),
            },
        }
    }

    fn evaluate_literal(&self, literal: &Literal) -> Result<Value, RuntimeError> {
        match literal {
            Literal::Null => Ok(Value::Null),
            Literal::Boolean(value) => Ok(Value::Boolean(*value)),
            Literal::Integer(value) => Ok(Value::Integer(*value)),
            Literal::Float(value) => Ok(Value::Float(*value)),
            Literal::String(value) => Ok(Value::String(value.clone())),
        }
    }
}

fn ensure_scope(context: &mut RuntimeContext<'_>) {
    if context.environment.scopes.is_empty() {
        context.environment.scopes.push(Default::default());
    }
}

fn set_variable(context: &mut RuntimeContext<'_>, name: &str, value: Value) {
    ensure_scope(context);
    context
        .environment
        .scopes
        .last_mut()
        .expect("scope exists")
        .values
        .insert(name.to_string(), value);
}

fn resolve_variable(context: &RuntimeContext<'_>, name: &str) -> Result<Value, RuntimeError> {
    for scope in context.environment.scopes.iter().rev() {
        if let Some(value) = scope.values.get(name) {
            return Ok(value.clone());
        }
    }
    Err(RuntimeError {
        message: format!("unknown variable `{name}`"),
    })
}

fn apply_add(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs + rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs + rhs)),
        (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs as f64 + rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Float(lhs + rhs as f64)),
        _ => Err(RuntimeError {
            message: "addition requires two numbers or two strings".to_string(),
        }),
    }
}

fn apply_subtract(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs - rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs - rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs as f64 - rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Float(lhs - rhs as f64)),
        _ => Err(RuntimeError {
            message: "subtraction requires numeric operands".to_string(),
        }),
    }
}

fn apply_multiply(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs * rhs)),
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs * rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs as f64 * rhs)),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Float(lhs * rhs as f64)),
        _ => Err(RuntimeError {
            message: "multiplication requires numeric operands".to_string(),
        }),
    }
}

fn apply_divide(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Integer(lhs), Value::Integer(rhs)) if rhs != 0 => Ok(Value::Integer(lhs / rhs)),
        (Value::Float(lhs), Value::Float(rhs)) if rhs != 0.0 => Ok(Value::Float(lhs / rhs)),
        (Value::Integer(lhs), Value::Float(rhs)) if rhs != 0.0 => {
            Ok(Value::Float(lhs as f64 / rhs))
        }
        (Value::Float(lhs), Value::Integer(rhs)) if rhs != 0 => Ok(Value::Float(lhs / rhs as f64)),
        _ => Err(RuntimeError {
            message: "division by zero or invalid operand types".to_string(),
        }),
    }
}

fn apply_modulo(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Integer(lhs), Value::Integer(rhs)) if rhs != 0 => Ok(Value::Integer(lhs % rhs)),
        (Value::Float(lhs), Value::Float(rhs)) if rhs != 0.0 => Ok(Value::Float(lhs % rhs)),
        _ => Err(RuntimeError {
            message: "modulo requires non-zero numeric operands".to_string(),
        }),
    }
}

fn compare<F>(left: Value, right: Value, cmp: F) -> Result<Value, RuntimeError>
where
    F: Fn(f64, f64) -> bool,
{
    match (left, right) {
        (Value::Integer(lhs), Value::Integer(rhs)) => {
            Ok(Value::Boolean(cmp(lhs as f64, rhs as f64)))
        }
        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Boolean(cmp(lhs, rhs))),
        (Value::Integer(lhs), Value::Float(rhs)) => Ok(Value::Boolean(cmp(lhs as f64, rhs))),
        (Value::Float(lhs), Value::Integer(rhs)) => Ok(Value::Boolean(cmp(lhs, rhs as f64))),
        _ => Err(RuntimeError {
            message: "comparison requires numeric operands".to_string(),
        }),
    }
}

// Helper serialization for return propagation. Keep this internal and simple.
fn serialize_value_for_return(value: &Value) -> String {
    match value {
        Value::Null => "N".to_string(),
        Value::Boolean(b) => format!("B:{}", if *b { 1 } else { 0 }),
        Value::Integer(i) => format!("I:{}", i),
        Value::Float(f) => format!("F:{}", f),
        Value::String(s) => format!("S:{}:{}", s.len(), s),
        Value::Function(_) => "X".to_string(),
        Value::Module(_) => "X".to_string(),
        Value::Array(_) => "X".to_string(),
        Value::Object(_) => "X".to_string(),
        Value::NativeFunction(_) => "X".to_string(),
    }
}

fn extract_return_from_error(err: &RuntimeError) -> Option<Value> {
    const PREFIX: &str = "__PCF_RETURN__:";
    if !err.message.starts_with(PREFIX) {
        return None;
    }
    let payload = &err.message[PREFIX.len()..];
    if payload == "N" {
        return Some(Value::Null);
    }
    if let Some(rest) = payload.strip_prefix("B:") {
        return Some(Value::Boolean(rest != "0"));
    }
    if let Some(rest) = payload.strip_prefix("I:") {
        if let Ok(i) = rest.parse::<i64>() {
            return Some(Value::Integer(i));
        }
    }
    if let Some(rest) = payload.strip_prefix("F:") {
        if let Ok(f) = rest.parse::<f64>() {
            return Some(Value::Float(f));
        }
    }
    if let Some(rest) = payload.strip_prefix("S:") {
        // format S:<len>:<data>
        if let Some(colon) = rest.find(':') {
            let (_len, data) = rest.split_at(colon + 1);
            return Some(Value::String(data.to_string()));
        }
    }
    // Non-serializable values are not supported for return propagation
    None
}
