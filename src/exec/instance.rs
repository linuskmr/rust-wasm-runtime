use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr};
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
				function: wasi::fd_write
			}
		];
		functions.extend(
			module.functions.wasm.into_iter()
				.map(|wasm_func| Callable::WasmFunction(wasm_func))
		);

		let memories = module.memory_blueprint.map(Memory::from);


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
	pub memory: &'a mut Option<Memory>,
	pub operand_stack: &'a mut Vec<Value>,
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

	#[tracing::instrument(skip(self))]
	fn exec_function(&mut self, function_index: usize) -> ExecutionResult {
		let function = self.functions.get(function_index)
			.ok_or(ExecutionError::FunctionIndexOutOfBounds {
				index: function_index,
				len: self.functions.len()
			})?;

		self.call_stack.push(function_index);
		tracing::trace!(callstack = ?self.call_stack().iter().map(|f| f.to_string()).collect::<Vec<_>>());

		// Execute function body
		match function {
			Callable::RustFunction { function, .. } => function(self)?,
			Callable::RustClosure { closure, .. } => closure(self)?,
			Callable::WasmFunction(function) => {
				self.execute_instructions(&function.body)?;
			},
		}

		self.call_stack.pop();
		Ok(())
	}

	fn execute_instructions<'iter>(&mut self, instructions: impl IntoIterator<Item=&'iter Instruction>) -> ExecutionResult {
		for instruction in instructions {
			let span = tracing::trace_span!("execute_instruction", ?instruction);
			let _span_enter = span.enter();
			match instruction {
				Instruction::Unreachable => return Err(ExecutionError::Trap("Instruction::Unreachable")),
				Instruction::Nop => (),
				Instruction::Block { block_type, instructions } => {
					self.execute_instructions(instructions)?;
				},
				Instruction::Loop { block_type, instructions } => {
					self.execute_instructions(instructions)?;
				},
				Instruction::If { block_type, if_instructions, else_instructions } => {
					let condition = i32::try_from(self.op_stack_pop()?)?;
					if condition != 0 {
						self.execute_instructions(if_instructions)?;
					} else {
						self.execute_instructions(else_instructions)?;
					}
				},
				Instruction::Return => break,
				Instruction::I32Const(val) => self.operand_stack.push(Value::I32(*val)),
				Instruction::I32Store(mem_arg) => {
					let val = i32::try_from(self.op_stack_pop()?)?;
					// Convert value to little endian, because memory is in little endian
					let val = val.to_le_bytes();

					let addr = i32::try_from(self.op_stack_pop()?)?;
					let addr = addr as usize + mem_arg.offset;
					let addr = addr..addr+4;

					tracing::trace!("mem[{:?}] <- {:?}", addr, val);
					let mem = self.memory.as_mut()
						.ok_or(ExecutionError::NoMemory)?;
					let mem_data_len = mem.data.len(); // Has to fetched in advance for borrow checker
					let mem_slice = mem.data.get_mut(addr.clone())
						.ok_or(ExecutionError::InvalidMemoryArea { addr, size: mem_data_len })?;
					mem_slice.copy_from_slice(&val);
				},
				Instruction::Call { function_index } => self.exec_function(*function_index)?,
				Instruction::Drop => { self.operand_stack.pop().ok_or(ExecutionError::PopOnEmptyOperandStack)?; },
				Instruction::I32Eqz => {
					let a = i32::try_from(self.op_stack_pop()?)?;
					let result = if a == 0 { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result));
				}
				Instruction::I32Eq => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = if lhs == rhs { 1 } else  { 0 };
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Add => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::wrapping_add(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Add => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::wrapping_sub(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Mul => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::wrapping_mul(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32DivU => {
					let lhs = u32::try_from(self.op_stack_pop()?)?;
					let rhs = u32::try_from(self.op_stack_pop()?)?;
					let result = u32::wrapping_div(lhs, rhs);
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32DivS => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::wrapping_div(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32RemU => {
					let lhs = u32::try_from(self.op_stack_pop()?)?;
					let rhs = u32::try_from(self.op_stack_pop()?)?;
					let result = u32::wrapping_rem(lhs, rhs);
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32RemS => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::wrapping_rem(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32And => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::bitand(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Or => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::bitor(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Xor => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::bitxor(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Shl => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::shl(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32ShrU => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::shr(lhs, rhs); // TODO: Possibly not correct sign extension?
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32ShrS => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::shr(lhs, rhs); // TODO: Possibly not correct sign extension?
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Rotr => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = i32::rotate_right(lhs, rhs as u32);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Clz => {
					let operand = i32::try_from(self.op_stack_pop()?)?;
					let result = operand.leading_zeros();
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32Ctz => {
					let operand = i32::try_from(self.op_stack_pop()?)?;
					let result = operand.trailing_zeros();
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32Popcnt => {
					let operand = i32::try_from(self.op_stack_pop()?)?;
					let result = operand.count_ones();
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32Ne => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = if lhs != rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32LtU => {
					let lhs = u32::try_from(self.op_stack_pop()?)?;
					let rhs = u32::try_from(self.op_stack_pop()?)?;
					let result = if lhs < rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32LtS => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = if lhs < rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GtU => {
					let lhs = u32::try_from(self.op_stack_pop()?)?;
					let rhs = u32::try_from(self.op_stack_pop()?)?;
					let result = if lhs > rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GtS => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = if lhs > rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32LeU => {
					let lhs = u32::try_from(self.op_stack_pop()?)?;
					let rhs = u32::try_from(self.op_stack_pop()?)?;
					let result = if lhs <= rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GtS => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = if lhs <= rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GeS => {
					let lhs = u32::try_from(self.op_stack_pop()?)?;
					let rhs = u32::try_from(self.op_stack_pop()?)?;
					let result = if lhs >= rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GeU => {
					let lhs = i32::try_from(self.op_stack_pop()?)?;
					let rhs = i32::try_from(self.op_stack_pop()?)?;
					let result = if lhs >= rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				_ => tracing::error!("unimplemented executing Instruction::{:?}", instruction),
			}
		}
		Ok(())
	}

	fn call_stack(&self) -> Vec<&Callable> {
		self.call_stack.iter()
			.map(|&function_index| &self.functions[function_index])
			.collect()
	}

	pub fn op_stack_pop(&mut self) -> Result<Value, ExecutionError> {
		self.operand_stack.pop().ok_or(ExecutionError::PopOnEmptyOperandStack)
	}
}
