use std::iter;
use std::fmt::Debug;
use std::ops::Range;
use thiserror::Error;
use crate::parse::{Function, Module};

/// Parsed instructions that can be used inside function bodies.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Instruction {
	/// No operation. Does exactly nothing.
	NoOp,
	/// Push a int32 onto the stack.
	ConstInt32(u8),
	/// Pop two int32 from the stack, add them and push the result onto the stack.
	AddInt32,
	/// Pop two int32 from the stack, subtract them and push the result onto the stack.
	SubInt32,
	// Pop two int32 from the stack, multiply them and push the result onto the stack.
	MulInt32,
	/// Increase memory size.
	IncreaseMem,
	/// Write uint8 to memory.
	StoreUint8,
	/// Call function by name
	FunctionCall(String),
	/// Debug stack and instruction pointer
	Debug,
}

impl Instruction {
	pub fn execute(&self, instance: &mut Instance) -> ExecutionResult {
		let runtime = &mut instance.runtime;
		match self {
			Instruction::NoOp => (),
			Instruction::ConstInt32(const_val) => {
				runtime.operand_stack.push(*const_val);
			},
			Instruction::AddInt32 => {
				let a = runtime.op_stack_pop()?;
				let b = runtime.op_stack_pop()?;
				runtime.operand_stack.push(a + b);
			},
			Instruction::SubInt32 => {
				let a = runtime.op_stack_pop()?;
				let b = runtime.op_stack_pop()?;
				runtime.operand_stack.push(a - b);
			},
			Instruction::MulInt32 => {
				let a = runtime.op_stack_pop()?;
				let b = runtime.op_stack_pop()?;
				runtime.operand_stack.push(a * b);
			},
			Instruction::IncreaseMem => {
				let amount = runtime.op_stack_pop()? as usize;
				log::debug!("Increase memory by {}", amount);
				runtime.mem.extend(iter::repeat(0u8).take(amount));
			},
			Instruction::StoreUint8 => {
				let val = runtime.op_stack_pop()?;
				let addr = runtime.op_stack_pop()? as usize;
				*runtime.mem_get_mut(addr)? = val;
			},
			Instruction::FunctionCall(name) => {
				instance.exec_function(name)?;
			}
			Instruction::Debug => {
				eprintln!("DEBUG {{");
				eprintln!("  Stack: {:?},", runtime.operand_stack);
				eprintln!("  Memory: {:?}", runtime.mem);
				eprintln!("}}");
			}
		}
		Ok(())
	}
}


pub type ExecutionResult = Result<(), ExecutionError>;

/// Something that can be called inside the context of a runtime. This is either a WebAssembly function or a
/// Rust function (used for extern functions like WASI).
///
/// Other ideas that would avoid this enum are:
/// * Implementing `Fn` for WebAssembly functions. However, implementing `Fn` for custom types is not stable yet.
/// * Creating a closure for all WebAssembly functions, which saves their instructions. This implies that every function
/// has to be `box`ed, which is inefficient.
pub enum Callable {
	WasmFunction(Function),
	RustClosure(Box<dyn Fn(&mut Instance) -> ExecutionResult>),
	RustFunction(fn(&mut Instance) -> ExecutionResult),
}

impl Callable {
	fn run(&self, instance: &mut Instance) -> ExecutionResult {
		match self {
			Callable::WasmFunction(function) => function.call(instance),
			Callable::RustClosure(closure) => closure(instance),
			Callable::RustFunction(function) => function(instance),
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub struct Runtime {
	/// Operand stack.
	pub(crate) operand_stack: Vec<u8>,
	/// Linear memory.
	pub(crate) mem: Vec<u8>,
}

impl Runtime {
	pub fn new() -> Self {
		Self {
			operand_stack: Vec::with_capacity(16),
			mem: Vec::new(),
		}
	}

	pub fn op_stack_pop(&mut self) -> Result<u8, ExecutionError> {
		self.operand_stack.pop().ok_or(ExecutionError::StackEmpty)
	}

	pub fn mem_slice(&mut self, index: Range<usize>) -> Result<&[u8], ExecutionError> {
		self.mem.get(index.clone()).ok_or(ExecutionError::MemRangeOutOfBounds { mem_size: self.mem.len(), range: index })
	}

	pub(crate) fn mem_get_mut(&mut self, index: usize) -> Result<&mut u8, ExecutionError> {
		// Avoid borrowing error in ok_or because of "trying to immutable borrow, but already mutable borrowed"
		let mem_len = self.mem.len();
		self.mem.get_mut(index).ok_or(ExecutionError::MemIndexOutOfBounds {mem_size: mem_len, index})
	}
}

/// A module in execution.
///
/// The borrows are needed, because otherwise the borrow checker is not smart enough to allow mutable access to the
/// runtime (&mut self: Instance) while having a immutable reference to module (&self: Instance). With the borrows,
/// a reference to a function from the module stays valid, even if the module is replaced with another by the mutable
/// borrow.
#[derive(Debug)]
pub struct Instance<'a> {
	pub module: &'a Module,
	pub runtime: &'a mut Runtime,
}

impl<'a> Instance<'a> {
	pub fn exec_function(&mut self, name: &str) -> Result<(), ExecutionError> {
		let function = self.module.functions.get(name)
			.ok_or(ExecutionError::UndefinedFunction(name.to_string()))?;
		function.run(self)?;
		Ok(())
	}
}

#[derive(Debug, Eq, PartialEq, Error)]
pub enum ExecutionError {
	#[error("ExecutionError::MemRangeOutOfBounds: Linear memory with size {mem_size} out of bounds for range {range:?}")]
	MemRangeOutOfBounds {
		mem_size: usize,
		range: Range<usize>
	},

	#[error("ExecutionError::MemIndexOutOfBounds: Linear memory with size {mem_size} out of bounds for index {index:?}")]
	MemIndexOutOfBounds {
		mem_size: usize,
		index: usize,
	},

	#[error("ExecutionError::StackEmpty: Operand stack unexpectedly empty")]
	StackEmpty,

	#[error("ExecutionError::UndefinedFunction: Call of undefined function {0}")]
	UndefinedFunction(String),
}