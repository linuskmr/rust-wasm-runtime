use crate::exec::memory::Memory;
use crate::exec::Value;

#[derive(Debug, PartialEq)]
pub struct Runtime {
	/// Operand stack.
	pub(crate) operand_stack: Vec<Value>,
	/// Linear memory.
	pub(crate) memories: Vec<Memory>,
}

impl Runtime {
	pub(crate) fn new(memories: Vec<Memory>) -> Runtime {
		let mut memories = memories;
		memories.iter_mut().for_each(|mem| mem.init());
		Self {
			operand_stack: Vec::new(),
			memories
		}
	}
}