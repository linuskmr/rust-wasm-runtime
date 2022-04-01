use crate::exec::memory::Memory;
use crate::exec::{Callable, Instruction, Value};
use crate::exec::error::ExecutionError;
use crate::parse::Module;

type ExecutionResult = Result<(), ExecutionError>;

/// A module in execution.
#[derive(Debug)]
pub struct Instance {
	functions: Vec<Callable>,
	memory: Option<Memory>,
	operand_stack: Vec<Value>,
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
			Callable::RustClosure {
				name: ("wasi_snapshot_preview1", "fd_write").into(),
				closure: Box::new(|| println!("fd_write called"))
			}
		];
		functions.extend(
			module.functions.wasm.into_iter()
				.map(|wasm_func| Callable::WasmFunction(wasm_func))
		);

		let memories = module.memory_blueprint.map(|mem_blueprint| Memory::from(mem_blueprint));


		Self { functions, memory: memories, operand_stack: Vec::new() }
	}

	fn as_ref(&mut self) -> InstanceRef {
		InstanceRef {
			functions: &self.functions,
			memory: &mut self.memory,
			operand_stack: &mut self.operand_stack
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
	memory: &'a mut Option<Memory>,
	operand_stack: &'a mut Vec<Value>,
}

impl<'a> InstanceRef<'a> {
	pub fn exec_start(&mut self) -> ExecutionResult {
		let start_function = self.functions.iter()
			.find(|func| {
				match func {
					Callable::WasmFunction(func) => {
						func.export_name.as_ref()
							.map(|export_name| export_name == "_start")
							.unwrap_or(false)
					},
					_ => false
				}
			}).expect("No start function");
		self.exec_function(start_function)
	}

	fn exec_function(&mut self, function: &Callable) -> ExecutionResult {
		match function {
			Callable::RustFunction { function, .. } => function(),
			Callable::RustClosure { closure, .. } => closure(),
			Callable::WasmFunction(function) => {
				for instruction in &function.body {
					self.exec_instruction(instruction)?;
				}
			}
		}
		Ok(())
	}

	fn exec_instruction(&mut self, instruction: &Instruction) -> ExecutionResult {
		log::trace!("executing instruction {:?}", instruction);
		match instruction {
			Instruction::I32Const(val) => self.operand_stack.push(Value::I32(*val)),
			Instruction::I32Store(mem_arg) => {
				let val = i32::try_from(self.operand_stack.pop().unwrap()).unwrap().to_le_bytes();
				let addr = i32::try_from(self.operand_stack.pop().unwrap()).unwrap();
				let addr = addr as usize + mem_arg.offset;
				log::trace!("mem[{}] = {:?}", addr, val);
				let mem = self.memory.as_mut().ok_or(ExecutionError::NoMemory)?;
				let mem_size = mem.data.len();
				let mem_slice = mem.data.get_mut(addr..addr+4).ok_or(ExecutionError::InvalidMemoryArea{ addr, size: mem_size })?;
				mem_slice.copy_from_slice(&val);
			},
			_ => log::error!("unimplemented executing instruction {:?}", instruction),
		}
		Ok(())
	}
}
