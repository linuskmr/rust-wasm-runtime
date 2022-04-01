use std::{io, string};
use thiserror::Error;
use num_enum::TryFromPrimitiveError;
use crate::parse::types::*;

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

	#[error("Unknown data mode: {0}")]
	UnknownDataMode(#[from] TryFromPrimitiveError<DataMode>),

	#[error("Function access out of range. index={index} wasm_len={wasm_len} imports_len={imports_len} total_len={total_len}")]
	WasmFunctionOutOfRange {
		index: usize,
		wasm_len: usize,
		imports_len: usize,
		total_len: usize
	},

	#[error("IoError: {0}")]
	IoError(#[from] io::Error),

	#[error("Expected opcode {0:?}")]
	ExpectedOpcode(Opcode),

	#[error("Leb128Error: {0}")]
	Leb128Error(#[from] leb128::read::Error),

	#[error("Utf8Error: {0}")]
	Utf8Error(#[from] string::FromUtf8Error),
}