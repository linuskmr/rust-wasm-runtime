use std::rc::Rc;
use crate::exec::error::ExecutionError;
use crate::exec::types::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
	V128,
	FuncRef,
	ExternRef,
	Function,
	Const,
	Var
}

impl TryFrom<Value> for i32 {
	type Error = ExecutionError;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::I32(val) => Ok(val),
			got => Err(ExecutionError::StackTypeError {
				got,
				expected: "i32",
			}),
		}
	}
}

impl TryFrom<Value> for u32 {
	type Error = ExecutionError;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::I32(val) => Ok(val as u32),
			got => Err(ExecutionError::StackTypeError {
				got,
				expected: "i32(u32)",
			}),
		}
	}
}

impl TryFrom<Value> for i64 {
	type Error = ExecutionError;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::I64(val) => Ok(val),
			got => Err(ExecutionError::StackTypeError {
				got,
				expected: "i64",
			}),
		}
	}
}

impl TryFrom<Value> for u64 {
	type Error = ExecutionError;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::I64(val) => Ok(val as u64),
			got => Err(ExecutionError::StackTypeError {
				got,
				expected: "i64(u64)",
			}),
		}
	}
}

impl TryFrom<Value> for usize {
	type Error = ExecutionError;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::I64(val) => Ok(val as usize),
			got => Err(ExecutionError::StackTypeError {
				got,
				expected: "usize",
			}),
		}
	}
}

impl Into<Value> for i32 {
	fn into(self) -> Value {
		Value::I32(self)
	}
}

impl Into<Value> for u32 {
	fn into(self) -> Value {
		Value::I32(self as i32)
	}
}

impl Into<Value> for i64 {
	fn into(self) -> Value {
		Value::I64(self)
	}
}

impl Into<Value> for u64 {
	fn into(self) -> Value {
		Value::I64(self as i64)
	}
}

impl Into<Value> for usize {
	fn into(self) -> Value {
		Value::I64(self as i64)
	}
}