use std::iter;
use std::ops::Range;
use thiserror::Error;
use crate::parse::{Function, Module};

/// Parsed instructions that can be used inside function bodies.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Instruction {
	Unreachable,
	Nop,
	Block { block_type: u8, instructions: Vec<Instruction> },
	Loop { block_type: u8, instructions: Vec<Instruction> },
	If { block_type: u8, if_instructions: Vec<Instruction>, else_instructions: Vec<Instruction> },
	Br { label_index: u8 },
	BrIf { label_index: u8 },
	BrTable { label_indexes: Vec<u8> },
	Return,
	Call { function_index: u8 },
	CallIndirect { table_index: u8, type_index: u8 },

	I32Const(i32),
	I64Const(i64),
	F32Const(f32),
	F64Const(f64),
	I32Eqz,
	I32Eq,
	I32Ne,
	I32LtS,
	I32LtU,
	I32GtS,
	I32GtU,
	I32LeS,
	I32LeU,
	I32GeS,
	I32GeU,

	I64Eqz,
	I64Eq,
	I64Ne,
	I64LtS,
	I64LtU,
	I64GtS,
	I64GtU,
	I64LeS,
	I64LeU,
	I64GeS,
	I64GeU,

	F32Eq,
	F32Ne,
	F32Lt,
	F32Gt,
	F32Le,
	F32Ge,

	F64Eq,
	F64Ne,
	F64Lt,
	F64Gt,
	F64Le,
	F64Ge,

	I32Clz,
	I32Ctz,
	I32Popcnt,
	I32Add,
	I32Sub,
	I32Mul,
	I32DivS,
	I32DivU,
	I32RemS,
	I32RemU,
	I32And,
	I32Or,
	I32Xor,
	I32Shl,
	I32ShrS,
	I32ShrU,
	I32Rotl,
	I32Rotr,

	I64Clz,
	I64Ctz,
	I64Popcnt,
	I64Add,
	I64Sub,
	I64Mul,
	I64DivS,
	I64DivU,
	I64RemS,
	I64RemU,
	I64And,
	I64Or,
	I64Xor,
	I64Shl,
	I64ShrS,
	I64ShrU,
	I64Rotl,
	I64Rotr,

	F32Abs,
	F32Neg,
	F32Ceil,
	F32Floor,
	F32Trunc,
	F32Nearest,
	F32Sqrt,
	F32Add,
	F32Sub,
	F32Mul,
	F32Div,
	F32Min,
	F32Max,
	F32Copysign,

	F64Abs,
	F64Neg,
	F64Ceil,
	F64Floor,
	F64Trunc,
	F64Nearest,
	F64Sqrt,
	F64Add,
	F64Sub,
	F64Mul,
	F64Div,
	F64Min,
	F64Max,
	F64Copysign,

	I32WrapI64,
	I32TruncF32S,
	I32TruncF32U,
	I32TruncF64S,
	I32TruncF64U,
	I64ExtendI32S,
	I64ExtendI32U,
	I64TruncF32S,
	I64TruncF32U,
	I64TruncF64S,
	I64TruncF64U,
	F32ConvertI32S,
	F32ConvertI32U,
	F32ConvertI64S,
	F32ConvertI64,
	F32DemoteF64,
	F64ConvertI32S,
	F64ConvertI32U,
	F64ConvertI64S,
	F64ConvertI64U,
	F64PromoteF32,
	I32ReinterpretF32,
	I64ReinterpretF64,
	F32ReinterpretI32,
	F64ReinterpretI64,

	I32Extend8S,
	I32Extend16S,
	I64Extend8S,
	I64Extend16S,
	I64Extend32S,
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

/// A module in execution. This instance contains a reference to a [Module] and a reference to [Runtime].
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