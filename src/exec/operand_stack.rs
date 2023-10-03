use crate::exec::error::Error;
use crate::exec::types;

/// The stack for working with values and instructions.
///
/// WebAssembly is a stack-based language, so values are pushed onto the operand stack,
/// and instructions pop values off the stack and the result onto the stack.
#[derive(Default, PartialEq, Debug, Clone)]
pub struct OperandStack(Vec<types::Value>);

impl OperandStack {
	/// Converts `value` into a [`Value`](types::Value) and pushes it onto the operand stack.
	pub fn push<T: Into<types::Value>>(&mut self, value: T) {
		self.0.push(value.into());
	}

	/// Pops a [`Value`](types::Value) off the operand stack and tries to convert in into a `T`.
	///
	/// If the stack is empty, an [`Error::PopOnEmptyOperandStack`] is returned.
	/// If the conversion fails, an [`Error::StackTypeError`] is returned.
	pub fn pop<T: TryFrom<types::Value>>(&mut self) -> Result<T, Error> {
		let value = self.0.pop().ok_or(Error::PopOnEmptyOperandStack)?;
		T::try_from(value.clone()).map_err(|_| Error::StackTypeError {
			got: value,
			expected: std::any::type_name::<T>(),
		})
	}
}