use std::convert::TryFrom;
use num_enum::TryFromPrimitive;
use std::iter::Peekable;
use std::{fmt, string};
use std::collections::HashMap;
use thiserror::Error;
use crate::exec::{Callable, Instance, ExecutionResult, Instruction};


/// All defined opcodes.
#[derive(Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
enum Opcode {
	/// No operation. Does exactly nothing.
	NoOp = 0,
	/// Push a int32 onto the stack.
	ConstInt32 = 1,
	/// Pop two int32 from the stack, add them and push the result onto the stack.
	AddInt32 = 2,
	// Pop two int32 from the stack, multiply them and push the result onto the stack.
	MulInt32 = 3,
	/// Increase memory size.
	IncreaseMem = 4,
	/// Write uint8 to memory.
	StoreUint8 = 5,
	/// Call function by name
	FunctionCall = 6,
	Function = 7,
	End = 8,
	/// Debug stack and instruction pointer
	Debug = 99,
}

/// A parsed WebAssembly module.
pub struct Module {
	pub functions: HashMap<String, Callable>,
}

impl Module {
	/// Parses `bytecode` into a [Module] or a [ParsingError].
	pub fn new(bytecode: impl IntoIterator<Item=u8>) -> Result<Module, ParsingError> {
		let parser = Parser { bytecode: bytecode.into_iter().peekable() };
		parser.parse_top_level()
	}
}

impl fmt::Debug for Module {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Todo: Use iter.intersperse.collect when it's stable
		let functions = self.functions.keys().cloned().collect::<Vec<String>>().join(", ");
		f.debug_struct("Module")
			.field("functions", &functions)
			.finish()
	}
}

/// A function which is declared and defined inside WebAssembly.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Function {
	/// Name of the function.
	pub name: String,
	/// Body with instructions.
	pub body: Vec<Instruction>,
}

impl Function {
	/// Convert this WebAssembly function to a callable rust function.
	pub fn call(&self, instance: &mut Instance) -> ExecutionResult {
		for instruction in &self.body {
			instruction.execute(instance)?;
		}
		Ok(())
	}
}

/// Parses bytecode into a WebAssembly AST ([Section]).
pub struct Parser<ByteIter: Iterator<Item=u8>> {
	bytecode: Peekable<ByteIter>
}

impl<ByteIter: Iterator<Item=u8>> Parser<ByteIter> {
	/// Peeks and parses the next opcode.
	fn peek_opcode(&mut self) -> Option<Result<Opcode, ParsingError>> {
		let opcode = *self.bytecode.peek()?;
		Some(match Opcode::try_from(opcode) {
			Ok(opcode) => Ok(opcode),
			Err(_) => Err(ParsingError::UnknownOpcode {opcode}),
		})
	}

	// Consumes and parses the next opcode.
	fn next_opcode(&mut self) -> Option<Result<Opcode, ParsingError>> {
		let opcode = self.bytecode.next()?;
		Some(match Opcode::try_from(opcode) {
			Ok(opcode) => Ok(opcode),
			Err(_) => Err(ParsingError::UnknownOpcode {opcode}),
		})
	}

	/// This top level parsing function parses the bytecode into a [Module].
	fn parse_top_level(mut self) -> Result<Module, ParsingError> {
		let mut module = Module { functions: HashMap::new() };
		loop {
			match self.next_opcode() {
				Some(Ok(Opcode::Function)) => {
					let wasm_function = self.parse_function()?;
					module.functions.insert(wasm_function.name.clone(), Callable::WasmFunction(wasm_function));
				},
				// Propagate unknown opcode error
				Some(Err(err)) => return Err(err),
				// Unexpected opcode
				Some(Ok(got)) => return Err(ParsingError::Expected {
					expected: "function",
					got: format!("{:?}", got)
				}),
				// End of file
				None => break
			};
		}
		Ok(module)
	}

	/// Parses a function.
	fn parse_function(&mut self) -> Result<Function, ParsingError> {
		// Parse null-terminated string
		let name: Vec<u8> = self.bytecode.by_ref()
			.take_while(|&byte| byte != 0)
			.collect();
		let name = String::from_utf8(name)?;

		// Parse function body
		let mut body = Vec::new();
		loop {
			let instruction = match self.parse_instruction() {
				Some(Ok(instruction)) => instruction,
				// Propagate error
				Some(Err(err)) => return Err(err),
				// No more body instructions. This will probably be an 'end' instruction.
				None => break,
			};
			body.push(instruction);
		}

		// Expect and consume end
		let end = self.next_opcode();
		if end != Some(Ok(Opcode::End)) {
			return Err(ParsingError::Expected {
				expected: "End",
				got: format!("{:?}", end),
			})
		}

		Ok(Function { name, body })
	}

	/// Parses normal instructions. If a special instruction like `end` is encountered, None will be returned.
	fn parse_instruction(&mut self) -> Option<Result<Instruction, ParsingError>> {
		let opcode = match self.peek_opcode()? {
			Ok(opcode) => opcode,
			Err(err) => return Some(Err(err)),
		};
		match opcode {
			Opcode::NoOp => {
				self.bytecode.next();
				Some(Ok(Instruction::NoOp))
			}
			Opcode::ConstInt32 => {
				self.bytecode.next();
				// Read the const value after the opcode
				let const_val = self.bytecode.next()
					.ok_or(ParsingError::Expected { expected: "const", got: String::from("None") });
				let const_val = match const_val {
					Err(err) => return Some(Err(err)),
					Ok(const_val) => const_val,
				};
				Some(Ok(Instruction::ConstInt32(const_val)))
			},
			Opcode::AddInt32 => {
				self.bytecode.next();
				Some(Ok(Instruction::AddInt32))
			},
			Opcode::MulInt32 => {
				self.bytecode.next();
				Some(Ok(Instruction::MulInt32))
			},
			Opcode::IncreaseMem => {
				self.bytecode.next();
				Some(Ok(Instruction::IncreaseMem))
			},
			Opcode::StoreUint8 => {
				self.bytecode.next();
				Some(Ok(Instruction::StoreUint8))
			},
			Opcode::Debug => {
				self.bytecode.next();
				Some(Ok(Instruction::Debug))
			}
			Opcode::FunctionCall => {
				self.bytecode.next();
				let name = self.bytecode.by_ref().take_while(|&byte| byte != 0).collect();
				let name = String::from_utf8(name)
					.map_err(|err| ParsingError::InvalidUtf8(err));
				let name = match name {
					Ok(name) => name,
					Err(err) => return Some(Err(err)),
				};
				Some(Ok(Instruction::FunctionCall(name)))
			}
			// This is not a invalid opcode, but a opcode that is not a "normal" instruction.
			// This might by something like an "end" instruction.
			_ => None,
		}
	}
}

#[derive(Debug, Eq, PartialEq, Error)]
pub enum ParsingError {
	#[error("ParsingError::UnknownOpcode: Unknown opcode {opcode:#x}")]
	UnknownOpcode {
		opcode: u8
	},

	#[error("ParsingError::InvalidUtf8: {0}")]
	InvalidUtf8(#[from] string::FromUtf8Error),

	#[error("ParsingError::Expected: expected={expected:?}, got={got:?}")]
	Expected {
		expected: &'static str,
		got: String,
	},
}