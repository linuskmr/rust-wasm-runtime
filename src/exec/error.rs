use std::io;
use std::ops::Range;
use thiserror::Error;
use crate::exec::Value;
use crate::parse::Type;

/// Execution errors.
#[derive(Debug, Error)]
pub enum Error {
	/// A memory instruction was called, but no memory is assigned to the module.
	#[error("A memory instruction was called, but no memory is assigned to the module")]
	NoMemory,

	/// Accessed address of memory with size.
	#[error("Accessed address {addr:?} of memory with size {size}")]
	InvalidMemoryArea {
		addr: Range<usize>,
		size: usize,
	},

	/// Function index out of bounds for length.
	#[error("Function index {index} out of bounds for length {len}")]
	FunctionIndexOutOfBounds {
		index: usize,
		len: usize,
	},

	/// Pop was called on an empty operand stack.
	#[error("Pop was called on an empty operand stack")]
	PopOnEmptyOperandStack,

	/// Expected on stack, got instead
	#[error("Expected {expected} on stack, got {got:?} instead")]
	StackTypeError {
		expected: &'static str,
		got: Value,
	},

	/// Trap because of...
	#[error("Trap because of {0}")]
	Trap(&'static str),

	/// Underlying IoError
	#[error("IoError: {0}")]
	IoError(#[from] io::Error),
}