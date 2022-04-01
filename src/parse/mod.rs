/*
use std::convert::TryFrom;
use std::{fmt, io, iter};
use std::fmt::{Formatter, Pointer};
use std::ops::Range;
use std::rc::Rc;

*/

// Export types so one can import only types without the rest of the module.
pub mod types;
// Only contains Parser, so re-export it in this module.
mod parser;
// Only contains ParsingError, so re-export in this module.
mod error;

pub use types::*;
pub use error::ParsingError;
pub use parser::Parser;

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