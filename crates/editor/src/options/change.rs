use super::{Error, ErrorKind, Name, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Set(Value),
    Toggle,
    Reset,
    CopyGlobal,
    Query,
    Add(i64),
    Subtract(i64),
    Multiply(i64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub name: Name,
    pub operation: Operation,
    pub text: String,
}

impl Change {
    pub fn value(&self, current: Value, global: Value) -> Result<Option<Value>, Error> {
        let value = match self.operation {
            Operation::Query => return Ok(None),
            Operation::Set(value) => value,
            Operation::Reset => self.name.default_value(),
            Operation::CopyGlobal => global,
            Operation::Toggle => match current {
                Value::Bool(value) => Value::Bool(!value),
                _ => return Err(self.error(ErrorKind::Trailing)),
            },
            Operation::Add(operand)
            | Operation::Subtract(operand)
            | Operation::Multiply(operand) => {
                let Value::Number(value) = current else {
                    return Err(self.error(ErrorKind::Invalid));
                };
                let number = match self.operation {
                    Operation::Add(_) => value.checked_add(operand),
                    Operation::Subtract(_) => value.checked_sub(operand),
                    _ => value.checked_mul(operand),
                }
                .ok_or_else(|| self.error(ErrorKind::Invalid))?;
                Value::Number(number)
            }
        };
        match (self.name.default_value(), value) {
            (Value::Bool(_), Value::Bool(_)) => {}
            (Value::Number(_), Value::Number(number))
                if !(i64::from(i32::MIN)..=i64::from(i32::MAX)).contains(&number) =>
            {
                return Err(self.error(ErrorKind::Invalid));
            }
            (Value::Number(_), Value::Number(number)) if number < 0 => {
                return Err(self.error(ErrorKind::Positive));
            }
            (Value::Number(_), Value::Number(_)) => {}
            _ => return Err(self.error(ErrorKind::Invalid)),
        }
        Ok(Some(value))
    }

    fn error(&self, kind: ErrorKind) -> Error {
        Error::new(kind, &self.text)
    }
}
