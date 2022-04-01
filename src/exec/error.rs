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
}