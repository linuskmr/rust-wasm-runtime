use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;
use crate::exec::error::ExecutionError;
use crate::exec::instance::InstanceRef;
use crate::parse::{ParsingError, Type};

pub(crate) type ExecutionResult = Result<(), ExecutionError>;

#[derive(PartialEq, Debug, Clone)]
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
	Call { function_index: usize },
	CallIndirect { table_index: usize, type_index: usize },

	RefNull,
	RefIsNull,
	RefFunc,

	Drop,
	Select,
	SelectValueType,

	LocalGet(usize),
	LocalSet(usize),
	LocalTee(usize),

	GlobalGet(usize),
	GlobalSet(usize),

	TableGet(usize),
	TableSet(usize),
	Extension,

	I32Load(MemArg),
	I64Load(MemArg),
	F32Load(MemArg),
	F64Load(MemArg),
	I32Load8s(MemArg),
	I32Load8u(MemArg),
	I32Load16s(MemArg),
	I32Load16u(MemArg),
	I64Load8s(MemArg),
	I64Load8u(MemArg),
	I64Load16s(MemArg),
	I66Load16u(MemArg),
	I64Load32s(MemArg),
	I64Load32u(MemArg),
	I32Store(MemArg),
	I64Store(MemArg),
	F32Store(MemArg),
	F64Store(MemArg),
	I32Store8(MemArg),
	I32Store16(MemArg),
	I64Store8(MemArg),
	I64Store16(MemArg),
	I64Store32(MemArg),


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
	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::I32(val) => Ok(val),
			_ => Err(()),
		}
	}
}

impl TryFrom<Value> for u32 {
	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::I32(val) => Ok(val as u32),
			_ => Err(()),
		}
	}
}

#[derive(Eq, PartialEq, Debug, Default, Clone)]
pub struct FunctionSignature {
	pub params: Vec<Type>,
	pub results: Vec<Type>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Identifier {
	pub(crate) module: String,
	pub(crate) field: String,
}

impl From<(&'static str, &'static str)> for Identifier {
	fn from(identifier: (&'static str, &'static str)) -> Self {
		Self {
			module: identifier.0.to_owned(),
			field: identifier.1.to_owned()
		}
	}
}

impl fmt::Display for Identifier {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}.{}", self.module, self.field)
	}
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct WasmFunction {
	pub export_name: Option<String>,
	pub signature: Rc<FunctionSignature>,
	pub locals: Vec<Type>,
	pub body: Vec<Instruction>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct MemArg {
	pub(crate) align: usize,
	pub(crate) offset: usize,
}

/// Something that can be called inside the context of a runtime. This is either a WebAssembly function or a
/// Rust function (used for extern functions like WASI).
///
/// Other ideas that would avoid this enum are:
/// * Implementing `Fn` for WebAssembly functions. However, implementing `Fn` for custom types is not stable yet.
/// * Creating a closure for all WebAssembly functions, which saves their instructions. This implies that every function
/// has to be `box`ed, which is inefficient.
pub enum Callable {
	WasmFunction(WasmFunction),
	RustClosure {
		name: Identifier,
		closure: Box<dyn Fn(&mut InstanceRef) -> ExecutionResult>
	},
	RustFunction {
		name: Identifier,
		function: fn(&mut InstanceRef) -> ExecutionResult
	},
}

impl fmt::Debug for Callable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Callable::WasmFunction(function) => function.fmt(f),
			Callable::RustFunction { name, .. } => {
				f.debug_struct("RustFunction")
					.field("name", name)
					.field("function", &"<opaque>")
					.finish()
			},
			Callable::RustClosure { name, .. } => {
				f.debug_struct("RustClosure")
					.field("name", name)
					.field("closure", &"<opaque>")
					.finish()
			},
		}
	}
}

impl fmt::Display for Callable {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Callable::WasmFunction(function) => {
				match &function.export_name {
					Some(name) => write!(f, "{}", name),
					None => write!(f, "None")
				}
			},
			Callable::RustFunction { name, .. } => write!(f, "{}", name),
			Callable::RustClosure { name, .. } => write!(f, "{}", name),
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExternFunction {
	pub(crate) name: Identifier,
	pub(crate) signature: Rc<FunctionSignature>,
}

#[derive(Default, Debug, PartialEq)]
pub struct Functions {
	pub(crate) imports: Vec<ExternFunction>,
	pub(crate) wasm: Vec<WasmFunction>,
}

impl Functions {
	pub(crate) fn get_wasm_function(&mut self, function_index: usize) -> Result<&mut WasmFunction, ParsingError> {
		let wasm_len = self.wasm.len();
		let imports_len = self.imports.len();
		let total_len = wasm_len + imports_len;

		let function_index = function_index.checked_sub(self.imports.len())
			.ok_or(ParsingError::WasmFunctionOutOfRange { index: function_index, wasm_len, imports_len, total_len })?;
		self.wasm.get_mut(function_index)
			.ok_or(ParsingError::WasmFunctionOutOfRange { index: function_index,  wasm_len, imports_len, total_len })
	}
}