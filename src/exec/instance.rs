use std::ops::{BitAnd, BitOr, BitXor, Deref, Shl, Shr};
use std::rc::Rc;
use crate::exec::memory::Memory;
use crate::exec::{Callable, Instruction, Value, ExecutionResult, wasi};
use crate::exec::error::ExecutionError;
use crate::exec::operand_stack::OperandStack;
use crate::parse::Module;


/// A module in execution.
#[derive(Debug)]
pub struct Instance {
	functions: Vec<Rc<Callable>>,
	memory: Option<Memory>,
	/// The stack for working with values and instructions.
	operand_stack: OperandStack,
	/// The function call stack, usually starting with `_start`.
	///
	/// You may visualize this using:
	/// `self.call_stack.iter().map(ToString::to_string).collect::<Vec<_>>()`
	call_stack: Vec<Rc<Callable>>,
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

		let mut functions: Vec<Rc<Callable>> = vec![
			Rc::new(Callable::RustFunction {
				name: ("wasi_snapshot_preview1", "fd_write").into(),
				function: wasi::fd_write
			})
		];
		functions.extend(
			module.functions.wasm.into_iter()
				.map(|wasm_func| Rc::new(Callable::WasmFunction(wasm_func)))
		);

		let memories = module.memory_blueprint.map(Memory::from);


		Self { functions, memory: memories, operand_stack: OperandStack::default(), call_stack: Vec::new() }
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

	pub fn operand_stack(&self) -> &OperandStack {
		&self.operand_stack
	}

	pub fn memory(&self) -> &Option<Memory> {
		&self.memory
	}
}

#[derive(Debug)]
pub struct InstanceRef<'a> {
	functions: &'a Vec<Rc<Callable>>,
	pub memory: &'a mut Option<Memory>,
	pub operand_stack: &'a mut OperandStack,
	call_stack: &'a mut Vec<Rc<Callable>>,
}

impl<'a> InstanceRef<'a> {
	pub fn exec_start(&mut self) -> ExecutionResult {
		// Search start function
		let (index, _function) = self.functions.iter()
			.enumerate()
			.find(|(_, func)| {
				match func.deref().deref() {
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

		self.call_stack.push(Rc::clone(&function));
		tracing::trace!(callstack = ?self.call_stack.iter().map(ToString::to_string).collect::<Vec<_>>());

		// Execute function body
		match function.deref().deref() {
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
					let condition = self.operand_stack.pop::<i32>()?;
					if condition != 0 {
						self.execute_instructions(if_instructions)?;
					} else {
						self.execute_instructions(else_instructions)?;
					}
				},
				Instruction::Return => break,
				Instruction::I32Const(val) => self.operand_stack.push(Value::I32(*val)),
				Instruction::I32Store(mem_arg) => {
					let val = self.operand_stack.pop::<i32>()?;
					// Convert value to little endian, because memory is in little endian
					let val = val.to_le_bytes();

					let addr = self.operand_stack.pop::<i32>()?;
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
				Instruction::Drop => { self.operand_stack.pop::<Value>()?; },
				Instruction::I32Eqz => {
					let a = self.operand_stack.pop::<i32>()?;
					let result = if a == 0 { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result));
				}
				Instruction::I32Eq => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = if lhs == rhs { 1 } else  { 0 };
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Add => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::wrapping_add(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Add => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::wrapping_sub(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Mul => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::wrapping_mul(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32DivU => {
					let lhs = self.operand_stack.pop::<u32>()?;
					let rhs = self.operand_stack.pop::<u32>()?;
					let result = u32::wrapping_div(lhs, rhs);
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32DivS => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::wrapping_div(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32RemU => {
					let lhs = self.operand_stack.pop::<u32>()?;
					let rhs = self.operand_stack.pop::<u32>()?;
					let result = u32::wrapping_rem(lhs, rhs);
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32RemS => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::wrapping_rem(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32And => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::bitand(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Or => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::bitor(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Xor => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::bitxor(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Shl => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::shl(lhs, rhs);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32ShrU => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::shr(lhs, rhs); // TODO: Possibly not correct sign extension?
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32ShrS => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::shr(lhs, rhs); // TODO: Possibly not correct sign extension?
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Rotr => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = i32::rotate_right(lhs, rhs as u32);
					self.operand_stack.push(Value::I32(result));
				},
				Instruction::I32Clz => {
					let operand = self.operand_stack.pop::<i32>()?;
					let result = operand.leading_zeros();
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32Ctz => {
					let operand = self.operand_stack.pop::<i32>()?;
					let result = operand.trailing_zeros();
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32Popcnt => {
					let operand = self.operand_stack.pop::<i32>()?;
					let result = operand.count_ones();
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32Ne => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = if lhs != rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32LtU => {
					let lhs = self.operand_stack.pop::<u32>()?;
					let rhs = self.operand_stack.pop::<u32>()?;
					let result = if lhs < rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32LtS => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = if lhs < rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GtU => {
					let lhs = self.operand_stack.pop::<u32>()?;
					let rhs = self.operand_stack.pop::<u32>()?;
					let result = if lhs > rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GtS => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = if lhs > rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32LeU => {
					let lhs = self.operand_stack.pop::<u32>()?;
					let rhs = self.operand_stack.pop::<u32>()?;
					let result = if lhs <= rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GtS => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = if lhs <= rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GeS => {
					let lhs = self.operand_stack.pop::<u32>()?;
					let rhs = self.operand_stack.pop::<u32>()?;
					let result = if lhs >= rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				Instruction::I32GeU => {
					let lhs = self.operand_stack.pop::<i32>()?;
					let rhs = self.operand_stack.pop::<i32>()?;
					let result = if lhs >= rhs { 1 } else { 0 };
					self.operand_stack.push(Value::I32(result as i32));
				},
				_ => tracing::error!("unimplemented executing Instruction::{:?}", instruction),
			}
		}
		Ok(())
	}
}
