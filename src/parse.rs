use std::collections::HashMap;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use std::convert::TryFrom;
use std::{fmt, io, iter, string};
use std::fmt::Formatter;
use std::ops::Range;
use std::rc::Rc;
use thiserror::Error;


/// https://webassembly.github.io/spec/core/binary/modules.html#sections
#[derive(Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum SectionId {
	Custom = 0,
	Type = 1,
	Import = 2,
	Function = 3,
	Table = 4,
	Memory = 5,
	Global = 6,
	Export = 7,
	Start = 8,
	Element = 9,
	Code = 10,
	Data = 11,
	DataCount = 12,
}

/// https://webassembly.github.io/spec/core/binary/instructions.html
#[derive(Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
	Unreachable          = 0x00,
	Nop                  = 0x01,
	Block                = 0x02,
	Loop                 = 0x03,
	If                   = 0x04,
	End                  = 0x0B,
	Br                   = 0x0C,
	BrIf                 = 0x0D,
	BrTable              = 0x0E,
	Return               = 0x0F,
	Call                 = 0x10,
	CallIndirect         = 0x11,
	RefNull              = 0xD0,
	RefIsNull            = 0xD1,
	RefFunc              = 0xD2,
	Drop                 = 0x1A,
	Select               = 0x1B,
	SelectValueType      = 0x1C,
	LocalGet             = 0x20,
	LocalSet             = 0x21,
	LocalTee             = 0x22,
	GlobalGet            = 0x23,
	GlobalSet            = 0x24,
	TableGet             = 0x25,
	TableSet             = 0x26,
	Extension            = 0xFC,
	I32Load              = 0x28,
	I64Load              = 0x29,
	F32Load              = 0x2A,
	F64Load              = 0x2B,
	I32Load8s            = 0x2C,
	I32Load8u            = 0x2D,
	I32Load16s           = 0x2E,
	I32Load16u           = 0x2F,
	I64Load8s            = 0x30,
	I64Load8u            = 0x31,
	I64Load16s           = 0x32,
	I66Load16u           = 0x33,
	I64Load32s           = 0x34,
	I64Load32u           = 0x35,
	I32Store             = 0x36,
	I64Store             = 0x37,
	F32Store             = 0x38,
	F64Store             = 0x39,
	I32Store8            = 0x3A,
	I32Store16           = 0x3B,
	I64Store8            = 0x3C,
	I64Store16           = 0x3D,
	I64Store32           = 0x3E,
	MemorySize           = 0x3F,
	MemoryGrow           = 0x40,
	I32Const             = 0x41,
	I64Const             = 0x42,
	F32Const             = 0x43,
	F64Const             = 0x44,
	I32Eqz               = 0x45,
	I32Eq                = 0x46,
	I32Ne                = 0x47,
	I32LtS               = 0x48,
	I32LtU               = 0x49,
	I32GtS               = 0x4A,
	I32GtU               = 0x4B,
	I32LeS               = 0x4C,
	I32LeU               = 0x4D,
	I32GeS               = 0x4E,
	I32GeU               = 0x4F,
	I64Eqz               = 0x50,
	I64Eq                = 0x51,
	I64Ne                = 0x52,
	I64LtS               = 0x53,
	I64LtU               = 0x54,
	I64GtS               = 0x55,
	I64GtU               = 0x56,
	I64LeS               = 0x57,
	I64LeU               = 0x58,
	I64GeS               = 0x59,
	I64GeU               = 0x5A,
	F32Eq                = 0x5B,
	F32Ne                = 0x5C,
	F32Lt                = 0x5D,
	F32Gt                = 0x5E,
	F32Le                = 0x5F,
	F32Ge                = 0x60,
	F64Eq                = 0x61,
	F64Ne                = 0x62,
	F64Lt                = 0x63,
	F64Gt                = 0x64,
	F64Le                = 0x65,
	F64Ge                = 0x66,
	I32Clz               = 0x67,
	I32Ctz               = 0x68,
	I32Popcnt            = 0x69,
	I32Add               = 0x6A,
	I32Sub               = 0x6B,
	I32Mul               = 0x6C,
	I32DivS              = 0x6D,
	I32DivU              = 0x6E,
	I32RemS              = 0x6F,
	I32RemU              = 0x70,
	I32And               = 0x71,
	I32Or                = 0x72,
	I32Xor               = 0x73,
	I32Shl               = 0x74,
	I32ShrS              = 0x75,
	I32ShrU              = 0x76,
	I32Rotl              = 0x77,
	I32Rotr              = 0x78,
	I64Clz               = 0x79,
	I64Ctz               = 0x7A,
	I64Popcnt            = 0x7B,
	I64Add               = 0x7C,
	I64Sub               = 0x7D,
	I64Mul               = 0x7E,
	I64DivS              = 0x7F,
	I64DivU              = 0x80,
	I64RemS              = 0x81,
	I64RemU              = 0x82,
	I64And               = 0x83,
	I64Or                = 0x84,
	I64Xor               = 0x85,
	I64Shl               = 0x86,
	I64ShrS              = 0x87,
	I64ShrU              = 0x88,
	I64Rotl              = 0x89,
	I64Rotr              = 0x8A,
	F32Abs               = 0x8B,
	F32Neg               = 0x8C,
	F32Ceil              = 0x8D,
	F32Floor             = 0x8E,
	F32Trunc             = 0x8F,
	F32Nearest           = 0x90,
	F32Sqrt              = 0x91,
	F32Add               = 0x92,
	F32Sub               = 0x93,
	F32Mul               = 0x94,
	F32Div               = 0x95,
	F32Min               = 0x96,
	F32Max               = 0x97,
	F32Copysign          = 0x98,
	F64Abs               = 0x99,
	F64Neg               = 0x9A,
	F64Ceil              = 0x9B,
	F64Floor             = 0x9C,
	F64Trunc             = 0x9D,
	F64Nearest           = 0x9E,
	F64Sqrt              = 0x9F,
	F64Add               = 0xA0,
	F64Sub               = 0xA1,
	F64Mul               = 0xA2,
	F64Div               = 0xA3,
	F64Min               = 0xA4,
	F64Max               = 0xA5,
	F64Copysign          = 0xA6,
	I32WrapI64           = 0xA7,
	I32TruncF32S         = 0xA8,
	I32TruncF32U         = 0xA9,
	I32TruncF64S         = 0xAA,
	I32TruncF64U         = 0xAB,
	I64ExtendI32S        = 0xAC,
	I64ExtendI32U        = 0xAD,
	I64TruncF32S         = 0xAE,
	I64TruncF32U         = 0xAF,
	I64TruncF64S         = 0xB0,
	I64TruncF64U         = 0xB1,
	F32ConvertI32S       = 0xB2,
	F32ConvertI32U       = 0xB3,
	F32ConvertI64S       = 0xB4,
	F32ConvertI64        = 0xB5,
	F32DemoteF64         = 0xB6,
	F64ConvertI32S       = 0xB7,
	F64ConvertI32U       = 0xB8,
	F64ConvertI64S       = 0xB9,
	F64ConvertI64U       = 0xBA,
	F64PromoteF32        = 0xBB,
	I32ReinterpretF32    = 0xBC,
	I64ReinterpretF64    = 0xBD,
	F32ReinterpretI32    = 0xBE,
	F64ReinterpretI64    = 0xBF,
	I32Extend8S          = 0xC0,
	I32Extend16S         = 0xC1,
	I64Extend8S          = 0xC2,
	I64Extend16S         = 0xC3,
	I64Extend32S         = 0xC4,
}


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

/// https://webassembly.github.io/spec/core/binary/types.html
#[derive(Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Type {
	I32 = 0x7F,
	I64 = 0x7E,
	F32 = 0x7D,
	F64 = 0x7C,
	V128 = 0x7B,
	FuncRef = 0x70,
	ExternRef = 0x6F,
	Function = 0x60,
	Const = 0x00,
	Var = 0x01,
}

/// https://webassembly.github.io/spec/core/binary/types.html#limits
#[derive(Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum LimitKind {
	Min = 0x00,
	MinMax = 0x01,
}

#[derive(Eq, PartialEq, Debug, Default)]
pub struct FunctionSignature {
	pub params: Vec<Type>,
	pub results: Vec<Type>,
}

/// https://webassembly.github.io/spec/core/binary/modules.html#export-section
#[derive(Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum ExportKind {
	Function = 0x00,
	Table = 0x01,
	Memory = 0x02,
	Global = 0x03,
}

#[derive(PartialEq, Debug, Default)]
pub struct Function {
	pub name: Option<String>,
	pub signature: Rc<FunctionSignature>,
	pub num_locals: usize,
	pub body: Vec<Instruction>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct MemArg {
	align: usize,
	offset: usize,
}

const MEMORY_PAGE_SIZE: usize = 4096;

#[derive(Default, PartialEq)]
pub struct Memory {
	data: Vec<u8>,
	/// Minimum and maximum page limit.
	limit: Range<usize>,
	name: Option<String>,
}

impl fmt::Debug for Memory {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		// Do not print self.data because it is very large
		f.debug_struct("Memory")
			.field("limit", &self.limit)
			.field("name", &self.name)
			.finish()
	}
}

impl Memory {
	fn grow(&mut self, new_page_size: usize) {
		assert!(new_page_size >= self.limit.start, "Memory grow too small");
		assert!(new_page_size <= self.limit.end, "Memory grow too large");

		log::debug!("Memory grow to {} pages", new_page_size);
		let new_byte_size = MEMORY_PAGE_SIZE * new_page_size;
		self.data.resize(new_byte_size, 0);
	}

	fn page_size(&self) -> usize {
		self.data.len() / MEMORY_PAGE_SIZE
	}
}

/// A parsed WebAssembly module.
#[derive(Default, Debug, PartialEq)]
pub struct Module {
	functions: Vec<Function>,
	memories: Vec<Memory>,
}

impl Module {
	/// Parses `bytecode` into a [Module] or a [ParsingError].
	pub fn new(bytecode: impl io::Read) -> Result<Module, ParsingError> {
		let parser = Parser {
			bytecode,
			module: Module::default(),
			types: Vec::new(),
		};
		parser.parse_module()
	}
}

pub struct Parser<ByteIter: io::Read> {
	types: Vec<Rc<FunctionSignature>>,
	module: Module,
	bytecode: ByteIter,
}

impl<ByteIter: io::Read> Parser<ByteIter> {
	/// Reads one byte from [self.bytecode].
	fn read_byte(&mut self) -> Result<u8, io::Error> {
		let mut buf = [0u8; 1];
		self.bytecode.read_exact(&mut buf)?;
		Ok(buf[0])
	}

	fn parse_function_type(&mut self) -> Result<FunctionSignature, ParsingError> {
		let mut function_type = FunctionSignature::default();
		if Type::try_from(self.read_byte()?)? != Type::Function {
			// TODO: Return error instead
			panic!("Illegal type for function");
		}

		{ // Parse params
			let num_params = leb128::read::unsigned(&mut self.bytecode)?;
			function_type.params.reserve_exact(num_params as usize);
			for _ in 0..num_params {
				let param_type = self.read_byte()?;
				let param_type = Type::try_from(param_type)?;
				function_type.params.push(param_type);
			}
		}

		{ // Parse results
			let num_results = leb128::read::unsigned(&mut self.bytecode)?;
			function_type.results.reserve_exact(num_results as usize);
			for _ in 0..num_results {
				let result_type = self.read_byte()?;
				let result_type = Type::try_from(result_type)?;
				function_type.results.push(result_type);
			}
		}

		Ok(function_type)
	}

	fn parse_type_section(&mut self) -> Result<Vec<Rc<FunctionSignature>>, ParsingError> {
		let num_types = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing type section with {} types", num_types);
		let mut types = Vec::with_capacity(num_types);
		for _ in 0..num_types {
			let function_type = self.parse_function_type()?;
			log::debug!("{:?}", function_type);
			types.push(Rc::new(function_type));
		}
		Ok(types)
	}

	fn parse_function_section(&mut self) -> Result<(), ParsingError> {
		let num_functions = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing function section with {} functions", num_functions);

		for _ in 0..num_functions {
			let function_type_index = leb128::read::unsigned(&mut self.bytecode)? as usize;
			let function = Function {
				name: None,
				signature: Rc::clone(&self.types[function_type_index]),
				..Function::default()
			};
			self.module.functions.push(function);
		}
		Ok(())
	}

	fn read_string(&mut self) -> Result<String, ParsingError> {
		let length = leb128::read::unsigned(&mut self.bytecode)? as usize;
		let mut string = vec![0u8; length];
		self.bytecode.read_exact(&mut string)?;
		let string = String::from_utf8(string)?;
		Ok(string)
	}

	fn parse_export(&mut self) -> Result<(), ParsingError> {
		let name = self.read_string()?;
		let kind = ExportKind::try_from(self.read_byte()?)?;
		let index = leb128::read::unsigned(&mut self.bytecode)? as usize;

		match kind {
			ExportKind::Function => {
				self.module.functions[index].name = Some(name);
			},
			ExportKind::Memory => {
				self.module.memories[index].name = Some(name);
			}
			_ => unimplemented!()
		}

		Ok(())
	}

	fn parse_export_section(&mut self) -> Result<(), ParsingError> {
		let num_exports = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing export section with {} functions", num_exports);

		for _ in 0..num_exports {
			let export = self.parse_export()?;
		}
		Ok(())
	}

	fn parse_block(&mut self) -> Result<Vec<Instruction>, ParsingError> {
		let instructions = self.parse_instructions()?;
		if Opcode::try_from(self.read_byte()?)? != Opcode::End {
			return Err(ParsingError::ExpectedOpcode(Opcode::End));
		}
		Ok(instructions)
	}

	fn parse_memarg(&mut self) -> Result<MemArg, ParsingError> {
		Ok(MemArg {
			align: leb128::read::unsigned(&mut self.bytecode)? as usize,
			offset: leb128::read::unsigned(&mut self.bytecode)? as usize,
		})
	}

	fn parse_instructions(&mut self) -> Result<Vec<Instruction>, ParsingError> {
		let mut instructions = Vec::new();
		loop {
			let opcode = Opcode::try_from(self.read_byte()?)?;
			let instruction = match opcode {
				Opcode::Unreachable => Instruction::Unreachable,
				Opcode::Nop => Instruction::Nop,
				Opcode::Block => Instruction::Block {
					instructions: self.parse_block()?,
					block_type: 0,
				},
				Opcode::End => break,
				Opcode::Return => Instruction::Return,
				Opcode::Call => {
					let function_index = leb128::read::unsigned(&mut self.bytecode)? as usize;
					Instruction::Call { function_index }
				},
				Opcode::CallIndirect => {
					let table_index = leb128::read::unsigned(&mut self.bytecode)? as usize;
					let type_index = leb128::read::unsigned(&mut self.bytecode)? as usize;
					Instruction::CallIndirect { table_index, type_index }
				}
				// ...
				Opcode::LocalGet => {
					let index = leb128::read::unsigned(&mut self.bytecode)? as usize;
					Instruction::LocalGet(index)
				},
				Opcode::LocalSet => {
					let index = leb128::read::unsigned(&mut self.bytecode)? as usize;
					Instruction::LocalSet(index)
				}
				Opcode::LocalTee => {
					let index = leb128::read::unsigned(&mut self.bytecode)? as usize;
					Instruction::LocalTee(index)
				}
				// ...
				Opcode::I32Load => Instruction::I32Load(self.parse_memarg()?),
				Opcode::I64Load => Instruction::I64Load(self.parse_memarg()?),
				Opcode::F32Load => Instruction::F32Load(self.parse_memarg()?),
				Opcode::F64Load => Instruction::F64Load(self.parse_memarg()?),
				Opcode::I32Load8s => Instruction::I32Load8s(self.parse_memarg()?),
				Opcode::I32Load8u => Instruction::I32Load8u(self.parse_memarg()?),
				Opcode::I32Load16s => Instruction::I32Load16s(self.parse_memarg()?),
				Opcode::I32Load16u => Instruction::I32Load16u(self.parse_memarg()?),
				Opcode::I64Load8s => Instruction::I64Load8s(self.parse_memarg()?),
				Opcode::I64Load8u => Instruction::I64Load8u(self.parse_memarg()?),
				Opcode::I64Load16s => Instruction::I64Load16s(self.parse_memarg()?),
				Opcode::I66Load16u => Instruction::I66Load16u(self.parse_memarg()?),
				Opcode::I64Load32s => Instruction::I64Load32s(self.parse_memarg()?),
				Opcode::I64Load32u => Instruction::I64Load32u(self.parse_memarg()?),
				Opcode::I32Store => Instruction::I32Store(self.parse_memarg()?),
				Opcode::I64Store => Instruction::I64Store(self.parse_memarg()?),
				Opcode::F32Store => Instruction::F32Store(self.parse_memarg()?),
				Opcode::F64Store => Instruction::F64Store(self.parse_memarg()?),
				Opcode::I32Store8 => Instruction::I32Store8(self.parse_memarg()?),
				Opcode::I32Store16 => Instruction::I32Store16(self.parse_memarg()?),
				Opcode::I64Store8 => Instruction::I64Store8(self.parse_memarg()?),
				Opcode::I64Store16 => Instruction::I64Store16(self.parse_memarg()?),
				Opcode::I64Store32 => Instruction::I64Store32(self.parse_memarg()?),
				Opcode::I32Const => {
					Instruction::I32Const(leb128::read::unsigned(&mut self.bytecode)? as i32)
				},
				Opcode::I64Const => {
					Instruction::I32Const(leb128::read::unsigned(&mut self.bytecode)? as i32)
				},
				Opcode::F32Const => {
					let mut float_bytes = [0u8; 4];
					self.bytecode.read_exact(&mut float_bytes)?;
					let float = f32::from_le_bytes(float_bytes);
					Instruction::F32Const(float)
				}
				Opcode::F64Const => {
					let mut float_bytes = [0u8; 8];
					self.bytecode.read_exact(&mut float_bytes)?;
					let float = f64::from_le_bytes(float_bytes);
					Instruction::F64Const(float)
				}
				Opcode::I32Eqz => Instruction::I32Eqz,
				Opcode::I32Eq => Instruction::I32Eq,
				Opcode::I32Ne => Instruction::I32Ne,
				Opcode::I32LtS => Instruction::I32LtS,
				Opcode::I32LtU => Instruction::I32LtU,
				Opcode::I32GtS => Instruction::I32GtS,
				Opcode::I32GtU => Instruction::I32GtU,
				Opcode::I32LeS => Instruction::I32LeS,
				Opcode::I32LeU => Instruction::I32LeU,
				Opcode::I32GeS => Instruction::I32GeS,
				Opcode::I32GeU => Instruction::I32GeU,
				Opcode::I64Eqz => Instruction::I64Eqz,
				Opcode::I64Eq => Instruction::I64Eq,
				Opcode::I64Ne => Instruction::I64Ne,
				Opcode::I64LtS => Instruction::I64LtS,
				Opcode::I64LtU => Instruction::I64LtU,
				Opcode::I64GtS => Instruction::I64GtS,
				Opcode::I64GtU => Instruction::I64GtU,
				Opcode::I64LeS => Instruction::I64LeS,
				Opcode::I64LeU => Instruction::I64LeU,
				Opcode::I64GeS => Instruction::I64GeS,
				Opcode::I64GeU => Instruction::I64GeU,
				Opcode::F32Eq => Instruction::F32Eq,
				Opcode::F32Ne => Instruction::F32Ne,
				Opcode::F32Lt => Instruction::F32Lt,
				Opcode::F32Gt => Instruction::F32Gt,
				Opcode::F32Le => Instruction::F32Le,
				Opcode::F32Ge => Instruction::F32Ge,
				Opcode::F64Eq => Instruction::F64Eq,
				Opcode::F64Ne => Instruction::F64Ne,
				Opcode::F64Lt => Instruction::F64Lt,
				Opcode::F64Gt => Instruction::F64Gt,
				Opcode::F64Le => Instruction::F64Le,
				Opcode::F64Ge => Instruction::F64Ge,
				Opcode::I32Clz => Instruction::I32Clz,
				Opcode::I32Ctz => Instruction::I32Ctz,
				Opcode::I32Popcnt => Instruction::I32Popcnt,
				Opcode::I32Add => Instruction::I32Add,
				Opcode::I32Sub => Instruction::I32Sub,
				Opcode::I32Mul => Instruction::I32Mul,
				Opcode::I32DivS => Instruction::I32DivS,
				Opcode::I32DivU => Instruction::I32DivU,
				Opcode::I32RemS => Instruction::I32RemS,
				Opcode::I32RemU => Instruction::I32RemU,
				Opcode::I32And => Instruction::I32And,
				Opcode::I32Or => Instruction::I32Or,
				Opcode::I32Xor => Instruction::I32Xor,
				Opcode::I32Shl => Instruction::I32Shl,
				Opcode::I32ShrS => Instruction::I32ShrS,
				Opcode::I32ShrU => Instruction::I32ShrU,
				Opcode::I32Rotl => Instruction::I32Rotl,
				Opcode::I32Rotr => Instruction::I32Rotr,
				Opcode::I64Clz => Instruction::I64Clz,
				Opcode::I64Ctz => Instruction::I64Ctz,
				Opcode::I64Popcnt => Instruction::I64Popcnt,
				Opcode::I64Add => Instruction::I64Add,
				Opcode::I64Sub => Instruction::I64Sub,
				Opcode::I64Mul => Instruction::I64Mul,
				Opcode::I64DivS => Instruction::I64DivS,
				Opcode::I64DivU => Instruction::I64DivU,
				Opcode::I64RemS => Instruction::I64RemS,
				Opcode::I64RemU => Instruction::I64RemU,
				Opcode::I64And => Instruction::I64And,
				Opcode::I64Or => Instruction::I64Or,
				Opcode::I64Xor => Instruction::I64Xor,
				Opcode::I64Shl => Instruction::I64Shl,
				Opcode::I64ShrS => Instruction::I64ShrS,
				Opcode::I64ShrU => Instruction::I64ShrU,
				Opcode::I64Rotl => Instruction::I64Rotl,
				Opcode::I64Rotr => Instruction::I64Rotr,
				Opcode::F32Abs => Instruction::F32Abs,
				Opcode::F32Neg => Instruction::F32Neg,
				Opcode::F32Ceil => Instruction::F32Ceil,
				Opcode::F32Floor => Instruction::F32Floor,
				Opcode::F32Trunc => Instruction::F32Trunc,
				Opcode::F32Nearest => Instruction::F32Nearest,
				Opcode::F32Sqrt => Instruction::F32Sqrt,
				Opcode::F32Add => Instruction::F32Add,
				Opcode::F32Sub => Instruction::F32Sub,
				Opcode::F32Mul => Instruction::F32Mul,
				Opcode::F32Div => Instruction::F32Div,
				Opcode::F32Min => Instruction::F32Min,
				Opcode::F32Max => Instruction::F32Max,
				Opcode::F32Copysign => Instruction::F32Copysign,
				Opcode::F64Abs => Instruction::F64Abs,
				Opcode::F64Neg => Instruction::F64Neg,
				Opcode::F64Ceil => Instruction::F64Ceil,
				Opcode::F64Floor => Instruction::F64Floor,
				Opcode::F64Trunc => Instruction::F64Trunc,
				Opcode::F64Nearest => Instruction::F64Nearest,
				Opcode::F64Sqrt => Instruction::F64Sqrt,
				Opcode::F64Add => Instruction::F64Add,
				Opcode::F64Sub => Instruction::F64Sub,
				Opcode::F64Mul => Instruction::F64Mul,
				Opcode::F64Div => Instruction::F64Div,
				Opcode::F64Min => Instruction::F64Min,
				Opcode::F64Max => Instruction::F64Max,
				Opcode::F64Copysign => Instruction::F64Copysign,
				Opcode::I32WrapI64 => Instruction::I32WrapI64,
				Opcode::I32TruncF32S => Instruction::I32TruncF32S,
				Opcode::I32TruncF32U => Instruction::I32TruncF32U,
				Opcode::I32TruncF64S => Instruction::I32TruncF64S,
				Opcode::I32TruncF64U => Instruction::I32TruncF64U,
				Opcode::I64ExtendI32S => Instruction::I64ExtendI32S,
				Opcode::I64ExtendI32U => Instruction::I64ExtendI32U,
				Opcode::I64TruncF32S => Instruction::I64TruncF32S,
				Opcode::I64TruncF32U => Instruction::I64TruncF32U,
				Opcode::I64TruncF64S => Instruction::I64TruncF64S,
				Opcode::I64TruncF64U => Instruction::I64TruncF64U,
				Opcode::F32ConvertI32S => Instruction::F32ConvertI32S,
				Opcode::F32ConvertI32U => Instruction::F32ConvertI32U,
				Opcode::F32ConvertI64S => Instruction::F32ConvertI64S,
				Opcode::F32ConvertI64 => Instruction::F32ConvertI64,
				Opcode::F32DemoteF64 => Instruction::F32DemoteF64,
				Opcode::F64ConvertI32S => Instruction::F64ConvertI32S,
				Opcode::F64ConvertI32U => Instruction::F64ConvertI32U,
				Opcode::F64ConvertI64S => Instruction::F64ConvertI64S,
				Opcode::F64ConvertI64U => Instruction::F64ConvertI64U,
				Opcode::F64PromoteF32 => Instruction::F64PromoteF32,
				Opcode::I32ReinterpretF32 => Instruction::I32ReinterpretF32,
				Opcode::I64ReinterpretF64 => Instruction::I64ReinterpretF64,
				Opcode::F32ReinterpretI32 => Instruction::F32ReinterpretI32,
				Opcode::F64ReinterpretI64 => Instruction::F64ReinterpretI64,
				Opcode::I32Extend8S => Instruction::I32Extend8S,
				Opcode::I32Extend16S => Instruction::I32Extend16S,
				Opcode::I64Extend8S => Instruction::I64Extend8S,
				Opcode::I64Extend16S => Instruction::I64Extend16S,
				Opcode::I64Extend32S => Instruction::I64Extend32S,
				other => {
					log::error!("Unimplemented opcode {:?}", other);
					continue
				}
			};
			instructions.push(instruction);
		}
		Ok(instructions)
	}

	fn parse_function_code(&mut self, index: usize) -> Result<(), ParsingError> {
		let code_size = leb128::read::unsigned(&mut self.bytecode)? as usize;
		let num_locals = leb128::read::unsigned(&mut self.bytecode)? as usize;
		let body = self.parse_instructions()?;
		self.module.functions[index].num_locals = num_locals;
		self.module.functions[index].body = body;
		Ok(())
	}

	fn parse_code_section(&mut self) -> Result<(), ParsingError> {
		let num_functions = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing code section with {} functions", num_functions);

		for index in 0..num_functions {
			self.parse_function_code(index)?;
		}
		Ok(())
	}

	fn parse_import_section(&mut self) -> Result<(), ParsingError> {
		let num_imports = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing import section with {} imports", num_imports);
		for _ in 0..num_imports {
			let module_name = self.read_string()?;
			let field_name = self.read_string()?;
			let import_kind = ExportKind::try_from(self.read_byte()?)?;
			let signature_index = leb128::read::unsigned(&mut self.bytecode)? as usize;
			match import_kind {
				ExportKind::Function => {
					let import_function = Function {
						name: Some(format!("IMPORT:{}.{}", module_name, field_name)),
						signature: Rc::clone(&self.types[signature_index]),
						num_locals: 0,
						body: vec![]
					};
					log::debug!("Import {:?}", import_function);
					self.module.functions.push(import_function);
				},
				_ => unimplemented!(),
			}
		}
		Ok(())
	}

	fn parse_memory_section(&mut self) -> Result<(), ParsingError> {
		let num_mems = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing memory section with {} imports", num_mems);
		assert!(num_mems <= 1, "Only zero or one memory allowed"); // TODO: Return error
		for _ in 0..num_mems {
			let memory_limit_kind = LimitKind::try_from(self.read_byte()?)?;
			let memory_limit = match memory_limit_kind {
				LimitKind::Min => {
					let min = leb128::read::unsigned(&mut self.bytecode)? as usize;
					min..(u32::MAX as usize)
				},
				LimitKind::MinMax => {
					let min = leb128::read::unsigned(&mut self.bytecode)? as usize;
					let max = leb128::read::unsigned(&mut self.bytecode)? as usize;
					min..max
				}
			};
			let mut memory = Memory::default();
			memory.limit = memory_limit.clone();
			memory.grow(memory_limit.start);
			log::trace!("{:?}", memory);
			self.module.memories.push(memory);
		}
		Ok(())
	}

	pub fn parse_module(mut self) -> Result<Module, ParsingError> {
		let mut magic = [0u8; 4];
		self.bytecode.read_exact(&mut magic)?;
		if magic != [0x00, 0x61, 0x73, 0x6D] {
			return Err(ParsingError::NotAWasmModule);
		}

		let mut version = [0u8; 4];
		self.bytecode.read_exact(&mut version)?;
		if version != [0x01, 0x00, 0x00, 0x00] {
			return Err(ParsingError::IllegalVersion(version));
		}

		while let Ok(section_id) = self.read_byte() {
			let section_id = SectionId::try_from(section_id)?;
			let section_size = leb128::read::unsigned(&mut self.bytecode)?;
			log::trace!("Section {:?} with size {:?} bytes", section_id, section_size);
			match section_id {
				SectionId::Type => self.types = self.parse_type_section()?,
				SectionId::Function => self.parse_function_section()?,
				SectionId::Export => self.parse_export_section()?,
				SectionId::Code => self.parse_code_section()?,
				SectionId::Import => self.parse_import_section()?,
				SectionId::Memory => self.parse_memory_section()?,
				other => {
					log::error!("Unknown section {:?}. Ending parsing with Ok", other);
					break
				},
			}
			log::info!("{:#?}", self.module);
		}
		Ok(self.module)
	}
}

#[derive(Debug, Error)]
pub enum ParsingError {
	#[error("The module does not start with the magic constant 0x00 0x61 0x73 0x6D")]
	NotAWasmModule,

	#[error("The version {0:?} is not supported")]
	IllegalVersion([u8; 4]),

	#[error("Unknown section id: {0}")]
	UnknownSectionId(#[from] TryFromPrimitiveError<SectionId>),

	#[error("Unknown type: {0}")]
	UnknownType(#[from] TryFromPrimitiveError<Type>),

	#[error("Unknown export kind: {0}")]
	UnknownExport(#[from] TryFromPrimitiveError<ExportKind>),

	#[error("Unknown opcode: {0}")]
	UnknownOpcode(#[from] TryFromPrimitiveError<Opcode>),

	#[error("Unknown limit: {0}")]
	UnknownLimit(#[from] TryFromPrimitiveError<LimitKind>),

	#[error("IoError: {0}")]
	IoError(#[from] io::Error),

	#[error("Expected opcode {0:?}")]
	ExpectedOpcode(Opcode),

	#[error("Leb128Error: {0}")]
	Leb128Error(#[from] leb128::read::Error),

	#[error("Utf8Error: {0}")]
	Utf8Error(#[from] string::FromUtf8Error),
}

/*#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn type_section() {
		let wasm = [
			0x02, // num types
			// func type 0
			0x60, // func
			0x02, // num params
			0x7f, // i32
			0x7f, // i32
			0x01, // num results
			0x7f, // i32
			// func type 1
			0x60, // func
			0x01, // num params
			0x7f, // i32
			0x01, // num results
			0x7f, // i32
		];
		let mut parser = Parser { bytecode: wasm.as_slice() };
		let actual_type_section = parser.parse_type_section().unwrap();
		let expected_type_section = [
			FunctionSignature {
				params: vec![
					Type::I32,
					Type::I32,
				],
				results: vec![
					Type::I32,
				],
			},
			FunctionSignature {
				params: vec![
					Type::I32,
				],
				results: vec![
					Type::I32,
				],
			},
		];
		assert_eq!(actual_type_section, expected_type_section);
	}

	#[test]
	fn function_section() {
		let wasm = [
			0x02, // num functions
			0x00, // function type 0
			0x01, // function type 1
		];
		let mut parser = Parser { bytecode: wasm.as_slice() };
		let actual_function_section = parser.parse_function_section().unwrap();
		let expected_function_section = [0, 1];
		assert_eq!(actual_function_section, expected_function_section);
	}

	#[test]
	fn export_section() {
		let wasm = [
			0x01, // num exports
			0x06, // export name, string length of "addTwo"
			0x61, 0x64, 0x64, 0x54, 0x77, 0x6f, // export name "addTwo"
			0x00, // export kind
			0x00, // export func index
		];
		let mut parser = Parser { bytecode: wasm.as_slice() };
		let actual_export_section = parser.parse_export_section().unwrap();
		let expected_export_section = [
			Export {
				name: "addTwo".to_owned(),
				kind: ExportKind::Function,
				index: 0
			}
		];
		assert_eq!(actual_export_section, expected_export_section);
	}
}*/