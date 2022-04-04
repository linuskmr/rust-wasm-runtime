

use tracing::{debug_span, error, trace};
use crate::exec::memory::Memory;
use crate::exec::{Callable, Instruction, Value, ExecutionResult, wasi};
use crate::exec::error::ExecutionError;
use crate::parse::Module;


/// A module in execution.
#[derive(Debug)]
pub struct Instance {
	functions: Vec<Callable>,
	memory: Option<Memory>,
	operand_stack: Vec<Value>,
	call_stack: Vec<usize>,
}

impl Instance {
	pub fn new(module: Module) -> Self {
		/*let wasi = {
			let mut wasi: HashMap<Identifier, Callable> = HashMap::new();
			wasi.insert(
				Identifier {
					module: "wasi_snapshot_preview1".to_owned(),
					field: "fd_write".to_owned()
				},
				Callable::RustClosure(Box::new(|| println!("fd_write called")))
			);
			wasi
		};

		for import in module.functions.imports {
			wasi[import.name]
		}*/

		let mut functions: Vec<Callable> = vec![
			Callable::RustFunction {
				name: ("wasi_snapshot_preview1", "fd_write").into(),
				function: wasi::fd_write_
			}
		];
		functions.extend(
			module.functions.wasm.into_iter()
				.map(|wasm_func| Callable::WasmFunction(wasm_func))
		);

		let memories = module.memory_blueprint.map(|mem_blueprint| Memory::from(mem_blueprint));


		Self { functions, memory: memories, operand_stack: Vec::new(), call_stack: Vec::new() }
	}

	fn as_ref(&mut self) -> InstanceRef {
		InstanceRef {
			functions: &self.functions,
			memory: &mut self.memory,
			operand_stack: &mut self.operand_stack,
			call_stack: &mut self.call_stack,
		}
	}

	pub fn start(&mut self) -> Result<(), ExecutionError> {
		self.as_ref().exec_start()
	}

	pub fn operand_stack(&self) -> &Vec<Value> {
		&self.operand_stack
	}

	pub fn memory(&self) -> &Option<Memory> {
		&self.memory
	}
}

#[derive(Debug)]
pub struct InstanceRef<'a> {
	functions: &'a Vec<Callable>,
	pub(crate) memory: &'a mut Option<Memory>,
	pub(crate) operand_stack: &'a mut Vec<Value>,
	call_stack: &'a mut Vec<usize>,
}

impl<'a> InstanceRef<'a> {
	pub fn exec_start(&mut self) -> ExecutionResult {
		// Search start function
		let (index, _function) = self.functions.iter()
			.enumerate()
			.find(|(_, func)| {
				match func {
					Callable::WasmFunction(func) => {
						func.export_name.as_ref()
							.map(|export_name| export_name == "_start")
							.unwrap_or(false)
					},
					_ => false
				}
			}).expect("No start function");
		self.exec_function(index)
	}

	fn exec_function(&mut self, function_index: usize) -> ExecutionResult {
		let function = self.functions.get(function_index)
			.ok_or(ExecutionError::FunctionIndexOutOfBounds {
				index: function_index,
				len: self.functions.len()
			})?;
		self.call_stack.push(function_index);
		let _log_span = debug_span!("function", function_index, name = %function).entered();

		trace!("callstack {:?}", self.call_stack().iter().map(|f| f.to_string()).collect::<Vec<_>>());

		match function {
			Callable::RustFunction { function, .. } => function(self)?,
			Callable::RustClosure { closure, .. } => closure(self)?,
			Callable::WasmFunction(function) => {
				for instruction in &function.body {
					self.exec_instruction(instruction)?;
				}
			},
		}
		self.call_stack.pop();
		Ok(())
	}

	fn exec_instruction(&mut self, instruction: &Instruction) -> ExecutionResult {
		trace!("executing Instruction::{:?}", instruction);
		match instruction {
			Instruction::I32Const(val) => self.operand_stack.push(Value::I32(*val)),
			Instruction::I32Store(mem_arg) => {
				let val = i32::try_from(self.operand_stack.pop().unwrap()).unwrap().to_le_bytes();
				let addr = i32::try_from(self.operand_stack.pop().unwrap()).unwrap();
				let addr = addr as usize + mem_arg.offset;
				trace!("mem[{}] = {:?}", addr, val);
				let mem = self.memory.as_mut().ok_or(ExecutionError::NoMemory)?;
				let mem_size = mem.data.len();
				let mem_slice = mem.data.get_mut(addr..addr+4).ok_or(ExecutionError::InvalidMemoryArea{ addr, size: mem_size })?;
				mem_slice.copy_from_slice(&val);
			},
			Instruction::Call { function_index } => {
				self.exec_function(*function_index)?;
			},
			Instruction::Drop => { self.operand_stack.pop().ok_or(ExecutionError::PopOnEmptyOperandStack)?; },
 			_ => error!("unimplemented executing Instruction::{:?}", instruction),
		}
		Ok(())
	}

	fn call_stack(&self) -> Vec<&Callable> {
		self.call_stack.iter()
			.map(|&function_index| &self.functions[function_index])
			.collect()
	}

	fn op_stack_pop_i32(&mut self) -> Result<i32, ExecutionError> {
		match i32::try_from(self.operand_stack.pop().unwrap()) {
			Ok(i) => Ok(i),
			Err(_) => Err(ExecutionError::PopOnEmptyOperandStack)
		}
	}

	pub(crate) fn op_stack_pop_u32(&mut self) -> Result<u32, ExecutionError> {
		match u32::try_from(self.operand_stack.pop().unwrap()) {
			Ok(i) => Ok(i),
			Err(_) => Err(ExecutionError::PopOnEmptyOperandStack)
		}
	}
}
