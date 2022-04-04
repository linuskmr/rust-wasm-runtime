use std::io;
use thiserror::Error;


#[derive(Debug, Error)]
pub enum ExecutionError {
	#[error("A memory instruction was called, but no memory is assigned to the module")]
	NoMemory,

	#[error("Accessed address {addr} of memory with size {size}")]
	InvalidMemoryArea {
		addr: usize,
		size: usize,
	},

	#[error("Function index {index} out of bounds for length {len}")]
	FunctionIndexOutOfBounds {
		index: usize,
		len: usize,
	},

	#[error("Pop was called on an empty operand stack")]
	PopOnEmptyOperandStack,

	#[error("IoError: {0}")]
	IoError(#[from] io::Error),
}