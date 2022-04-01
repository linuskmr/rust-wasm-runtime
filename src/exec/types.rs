use std::{fmt, io};
use std::rc::Rc;
use crate::exec::Memory;
use crate::parse::{ParsingError, Type, Parser, MemoryBlueprint};

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

#[derive(Eq, PartialEq, Debug, Default, Clone)]
pub struct FunctionSignature {
	pub params: Vec<Type>,
	pub results: Vec<Type>,
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct Function {
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
	WasmFunction(Function),
	RustClosure(Box<dyn Fn()>),
	RustFunction(fn()),
}

impl fmt::Debug for Callable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Callable::WasmFunction(function) => function.fmt(f),
			_ => write!(f, "Extern rust function"),
		}
	}
}

/// A parsed WebAssembly module.
#[derive(Default, Debug)]
pub struct Module {
	pub(crate) functions: Vec<Callable>,
	pub(crate) memories: Vec<MemoryBlueprint>,
}

impl Module {
	/// Parses `bytecode` into a [Module] or a [ParsingError].
	pub fn new(bytecode: impl io::Read) -> Result<Module, ParsingError> {
		Parser::parse_module(bytecode)
	}
}

// This convenience function makes it possible to convert a `Callable` stored in `module.functions` to a `Function`,
// which is often needed.
impl<'a> TryFrom<&'a mut Callable> for &'a mut Function {
	type Error = ParsingError;

	fn try_from(callable: &'a mut Callable) -> Result<Self, Self::Error> {
		match callable {
			Callable::WasmFunction(function) => Ok(function),
			_ => Err(ParsingError::ModifyExternFunction),
		}
	}
}