use std::io;
use std::ops::Range;
use thiserror::Error;
use crate::exec::Value;
use crate::parse::Type;


#[derive(Debug, Error)]
pub enum ExecutionError {
	#[error("A memory instruction was called, but no memory is assigned to the module")]
	NoMemory,

	#[error("Accessed address {addr:?} of memory with size {size}")]
	InvalidMemoryArea {
		addr: Range<usize>,
		size: usize,
	},

	#[error("Function index {index} out of bounds for length {len}")]
	FunctionIndexOutOfBounds {
		index: usize,
		len: usize,
	},

	#[error("Pop was called on an empty operand stack")]
	PopOnEmptyOperandStack,
	
	#[error("Expected {expected} on stack, got {got:?} instead")]
	StackTypeError {
		expected: &'static str,
		got: Value,
	},
	
	#[error("Trap because of {0}")]
	Trap(&'static str),

	#[error("IoError: {0}")]
	IoError(#[from] io::Error),
}