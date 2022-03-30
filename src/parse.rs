use std::collections::HashMap;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use std::convert::TryFrom;
use std::io;
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
enum Opcode {
    Unreachable = 0x00,
    Nop = 0x01,
    Block = 0x02,
    Loop = 0x03,
    If = 0x04,
    Br = 0x0C,
    BrIf = 0x0D,
    BrTable = 0x0E,
    Return = 0x0F,
    Call = 0x10,
    CallIndirect = 0x11,

    RefNull = 0xD0,
    RefIsNull = 0xD1,
    RefFunc = 0xD2,

    Drop = 0x1A,
    Select = 0x1B,
    SelectValueType = 0x1C,

    LocalGet = 0x20,
    LocalSet = 0x21,
    LocalTee = 0x22,

    GlobalGet = 0x23,
    GlobalSet = 0x24,

    TableGet = 0x25,
    TableSet = 0x26,
    Extension = 0xFC,

    I32Load = 0x28,
    I64Load = 0x29,
    F32Load = 0x2A,
    F64Load = 0x2B,

    I32Load8s = 0x2C,
    I32Load8u = 0x2D,
    I32Load16s = 0x2E,
    I32Load16u = 0x2F,
    I64Load8s = 0x30,
    I64Load8u = 0x31,
    I64Load16s = 0x32,
    I66Load16u = 0x33,
    I64Load32s = 0x34,
    I64Load32u = 0x35,

    I32Store = 0x36,
    I64Store = 0x37,
    F32Store = 0x38,
    F64Store = 0x39,
    I32Store8 = 0x3A,
    I32Store16 = 0x3B,
    I64Store8 = 0x3C,
    I64Store16 = 0x3D,
    I64Store32 = 0x3E,

    MemorySize = 0x3F,
    MemoryGrow = 0x40,

    I32Const = 0x41,
    I64Const = 0x42,
    F32Const = 0x43,
    F64Const = 0x44,

    I32Eqz = 0x45,
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4A,
    I32GtU = 0x4B,
    I32LeS = 0x4C,
    I32LeU = 0x4D,
    I32GeS = 0x4E,
    I32GeU = 0x4F,

    I64Eqz = 0x50,
    I64Eq = 0x51,
    I64Ne = 0x52,
    I64LtS = 0x53,
    I64LtU = 0x54,
    I64GtS = 0x55,
    I64GtU = 0x56,
    I64LeS = 0x57,
    I64LeU = 0x58,
    I64GeS = 0x59,
    I64GeU = 0x5A,

    F32Eq = 0x5B,
    F32Ne = 0x5C,
    F32Lt = 0x5D,
    F32Gt = 0x5E,
    F32Le = 0x5F,
    F32Ge = 0x60,

    F64Eq = 0x61,
    F64Ne = 0x62,
    F64Lt = 0x63,
    F64Gt = 0x64,
    F64Le = 0x65,
    F64Ge = 0x66,

    I32Clz = 0x67,
    I32Ctz = 0x68,
    I32Popcnt = 0x69,
    I32Add = 0x6A,
    I32Sub = 0x6B,
    I32Mul = 0x6C,
    I32DivS = 0x6D,
    I32DivU = 0x6E,
    I32RemS = 0x6F,
    I32RemU = 0x70,
    I32And = 0x71,
    I32Or = 0x72,
    I32Xor = 0x73,
    I32Shl = 0x74,
    I32ShrS = 0x75,
    I32ShrU = 0x76,
    I32Rotl = 0x77,
    I32Rotr = 0x78,

    I64Clz = 0x79,
    I64Ctz = 0x7A,
    I64Popcnt = 0x7B,
    I64Add = 0x7C,
    I64Sub = 0x7D,
    I64Mul = 0x7E,
    I64DivS = 0x7F,
    I64DivU = 0x80,
    I64RemS = 0x81,
    I64RemU = 0x82,
    I64And = 0x83,
    I64Or = 0x84,
    I64Xor = 0x85,
    I64Shl = 0x86,
    I64ShrS = 0x87,
    I64ShrU = 0x88,
    I64Rotl = 0x89,
    I64Rotr = 0x8A,

    F32Abs = 0x8B,
    F32Neg = 0x8C,
    F32Ceil = 0x8D,
    F32Floor = 0x8E,
    F32Trunc = 0x8F,
    F32Nearest = 0x90,
    F32Sqrt = 0x91,
    F32Add = 0x92,
    F32Sub = 0x93,
    F32Mul = 0x94,
    F32Div = 0x95,
    F32Min = 0x96,
    F32Max = 0x97,
    F32Copysign = 0x98,

    F64Abs = 0x99,
    F64Neg = 0x9A,
    F64Ceil = 0x9B,
    F64Floor = 0x9C,
    F64Trunc = 0x9D,
    F64Nearest = 0x9E,
    F64Sqrt = 0x9F,
    F64Add = 0xA0,
    F64Sub = 0xA1,
    F64Mul = 0xA2,
    F64Div = 0xA3,
    F64Min = 0xA4,
    F64Max = 0xA5,
    F64Copysign = 0xA6,

    I32WrapI64 = 0xA7,
    I32TruncF32S = 0xA8,
    I32TruncF32U = 0xA9,
    I32TruncF64S = 0xAA,
    I32TruncF64U = 0xAB,
    I64ExtendI32S = 0xAC,
    I64ExtendI32U = 0xAD,
    I64TruncF32S = 0xAE,
    I64TruncF32U = 0xAF,
    I64TruncF64S = 0xB0,
    I64TruncF64U = 0xB1,
    F32ConvertI32S = 0xB2,
    F32ConvertI32U = 0xB3,
    F32ConvertI64S = 0xB4,
    F32ConvertI64 = 0xB5,
    F32DemoteF64 = 0xB6,
    F64ConvertI32S = 0xB7,
    F64ConvertI32U = 0xB8,
    F64ConvertI64S = 0xB9,
    F64ConvertI64U = 0xBA,
    F64PromoteF32 = 0xBB,
    I32ReinterpretF32 = 0xBC,
    I64ReinterpretF64 = 0xBD,
    F32ReinterpretI32 = 0xBE,
    F64ReinterpretI64 = 0xBF,

    I32Extend8S = 0xC0,
    I32Extend16S = 0xC1,
    I64Extend8S = 0xC2,
    I64Extend16S = 0xC3,
    I64Extend32S = 0xC4,
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

#[derive(Eq, PartialEq, Debug, Default)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub results: Vec<Type>,
}

/// A parsed WebAssembly module.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Module {
    /// A vector of function types that represent the types component of a module.
    types: Vec<FunctionType>,
    /// A vector of type indices that represent the type field of the functions in the funcs component of a module.
    function_types: Vec<usize>,
}

impl Module {
    /// Parses `bytecode` into a [Module] or a [ParsingError].
    pub fn new(bytecode: impl io::Read) -> Result<Module, ParsingError> {
        let parser = Parser { bytecode };
        parser.parse_module()
    }
}

pub struct Parser<ByteIter: io::Read> {
    bytecode: ByteIter,
}

impl<ByteIter: io::Read> Parser<ByteIter> {
    /// Reads one byte from [self.bytecode].
    fn read_byte(&mut self) -> Result<u8, io::Error> {
        let mut buf = [0u8; 1];
        self.bytecode.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn parse_function_type(&mut self) -> Result<FunctionType, ParsingError> {
        let mut function_type = FunctionType::default();
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

	fn parse_type_section(&mut self) -> Result<Vec<FunctionType>, ParsingError> {
		let num_types = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::debug!("Parsing type section with {} types", num_types);
		let mut types = Vec::with_capacity(num_types);
		for _ in 0..num_types {
			let function_type = self.parse_function_type()?;
			log::debug!("{:?}", function_type);
			types.push(function_type);
		}
		Ok(types)
	}

    fn parse_function_section(&mut self) -> Result<Vec<usize>, ParsingError> {
        let num_functions = leb128::read::unsigned(&mut self.bytecode)? as usize;
        log::debug!("Parsing function section with {} functions", num_functions);
        let mut functions = Vec::with_capacity(num_functions);
        for _ in 0..num_functions {
            let function_type_index = leb128::read::unsigned(&mut self.bytecode)? as usize;
            functions.push(function_type_index);
        }
        Ok(functions)
    }

    fn parse_module(mut self) -> Result<Module, ParsingError> {
        let mut module = Module::default();

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
            log::debug!("Section {:?} with size {:?} bytes", section_id, section_size);
            match section_id {
                SectionId::Type => module.types = self.parse_type_section()?,
                SectionId::Function => module.function_types = self.parse_function_section()?,
                _ => break,
            }
        }

       Ok(module)
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

    #[error("IoError: {0}")]
    IoError(#[from] io::Error),

    #[error("Leb128Error: {0}")]
    Leb128Error(#[from] leb128::read::Error),
}

#[cfg(test)]
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
            FunctionType {
                params: vec![
                    Type::I32,
                    Type::I32,
                ],
                results: vec![
                    Type::I32,
                ],
            },
            FunctionType {
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
        let expected_function_section = vec![0, 1];
        assert_eq!(actual_function_section, expected_function_section);
    }
}