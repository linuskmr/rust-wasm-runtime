use std::fmt;
use std::rc::Rc;
use crate::exec::instance::InstanceRef;
use crate::exec::types::*;
use crate::parse::{ParsingError, Type};

#[derive(Default, Debug, PartialEq)]
pub struct Functions {
	pub imports: Vec<ExternFunction>,
	pub wasm: Vec<WasmFunction>,
}

impl Functions {
	pub fn get_wasm_function(&mut self, function_index: usize) -> Result<&mut WasmFunction, ParsingError> {
		let wasm_len = self.wasm.len();
		let imports_len = self.imports.len();
		let total_len = wasm_len + imports_len;

		let function_index = function_index.checked_sub(self.imports.len())
			.ok_or(ParsingError::WasmFunctionOutOfRange { index: function_index, wasm_len, imports_len, total_len })?;
		self.wasm.get_mut(function_index)
			.ok_or(ParsingError::WasmFunctionOutOfRange { index: function_index,  wasm_len, imports_len, total_len })
	}
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

impl Callable {
	fn name(&self) -> String {
		match self {
			Callable::WasmFunction(function) => {
				match &function.export_name {
					Some(name) => name.clone(),
					None => format!("{}", function.index),
				}
			},
			Callable::RustFunction { name, .. } => name.to_string(),
			Callable::RustClosure { name, .. } => name.to_string(),
		}
	}
}

impl fmt::Display for Callable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name())
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExternFunction {
	pub name: Identifier,
	pub signature: Rc<FunctionSignature>,
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct WasmFunction {
	pub index: usize,
	pub export_name: Option<String>,
	pub signature: Rc<FunctionSignature>,
	pub locals: Vec<Type>,
	pub body: Vec<Instruction>,
}