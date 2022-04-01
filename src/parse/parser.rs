use std::{io, iter};
use std::rc::Rc;
use crate::parse::{
	error::*,
	types::*,
};
use crate::exec::{types::*};

pub struct Parser<ByteIter: io::Read> {
	types: Vec<Rc<FunctionSignature>>,
	/// This is the index from which `module.functions` contains `WasmFunction`s. All functions below this index are extern
	/// functions.
	wasm_functions_index: usize,
	module: Module,
	bytecode: ByteIter,
}

impl<ByteIter: io::Read> Parser<ByteIter> {
	pub fn parse_module(bytecode: ByteIter) -> Result<Module, ParsingError> {
		let parser = Parser {
			bytecode,
			module: Module::default(),
			wasm_functions_index: 0,
			types: Vec::new(),
		};
		parser.parse_module_internal()
	}

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
				export_name: None,
				signature: Rc::clone(&self.types[function_type_index]),
				..Function::default()
			};
			self.module.functions.push(Callable::WasmFunction(function));
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
				match &mut self.module.functions[index] {
					Callable::WasmFunction(function) => function.export_name = Some(name),
					_ => return Err(ParsingError::ModifyExternFunction),
				}
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
			self.parse_export()?;
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

	fn parse_locals(&mut self, function_index: usize) -> Result<(), ParsingError> {
		let num_locals = leb128::read::unsigned(&mut self.bytecode)? as usize;
		for _ in 0..num_locals {
			// A local declaration is a tuple of (local type count, local type)
			let num_locals_of_type = leb128::read::unsigned(&mut self.bytecode)? as usize;
			let local_type = Type::try_from(self.read_byte()?)?;
			let locals_of_type = iter::repeat(local_type).take(num_locals_of_type);
			<&mut Function>::try_from(&mut self.module.functions[function_index])?.locals.extend(locals_of_type);
		}
		Ok(())
	}

	fn parse_function_code(&mut self, function_index: usize) -> Result<(), ParsingError> {
		let _code_size = leb128::read::unsigned(&mut self.bytecode)? as usize;
		self.parse_locals(function_index)?;
		<&mut Function>::try_from(&mut self.module.functions[function_index])?.body = self.parse_instructions()?;
		Ok(())
	}

	fn parse_code_section(&mut self) -> Result<(), ParsingError> {
		let num_functions = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing code section with {} functions", num_functions);

		for i in 0..num_functions {
			// Skip extern functions when assigning code body to WASM functions
			let function_index = self.wasm_functions_index + i;
			self.parse_function_code(function_index)?;
		}
		Ok(())
	}

	fn parse_import_section(&mut self) -> Result<(), ParsingError> {
		let num_imports = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing import section with {} imports", num_imports);
		self.wasm_functions_index = num_imports;
		for _ in 0..num_imports {
			let module_name = self.read_string()?;
			let field_name = self.read_string()?;
			let import_kind = ExportKind::try_from(self.read_byte()?)?;
			let signature_index = leb128::read::unsigned(&mut self.bytecode)? as usize;
			match import_kind {
				ExportKind::Function => {
					let import_function = Function {
						export_name: Some(format!("IMPORT:{}.{}", module_name, field_name)),
						signature: Rc::clone(&self.types[signature_index]),
						locals: Vec::new(),
						body: Vec::new(),
					};
					log::debug!("Import {:?}", import_function);
					self.module.functions.push(Callable::WasmFunction(import_function));
				},
				_ => unimplemented!(),
			}
		}
		Ok(())
	}

	fn parse_memory_section(&mut self) -> Result<(), ParsingError> {
		let num_mems = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing memory section with {} imports", num_mems);
		for _ in 0..num_mems {
			let memory_limit_kind = LimitKind::try_from(self.read_byte()?)?;
			let page_limit = match memory_limit_kind {
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
			let memory_blueprint = MemoryBlueprint {
				page_limit,
				name: None
			};
			log::trace!("{:?}", memory_blueprint);
			self.module.memories.push(memory_blueprint);
		}
		Ok(())
	}

	fn parse_data_section(&mut self) -> Result<(), ParsingError> {
		let num_segments = leb128::read::unsigned(&mut self.bytecode)? as usize;
		log::trace!("Parsing data section with {} segments", num_segments);

		for _ in 0..num_segments {
			let data_mode = DataMode::try_from(self.read_byte()?)?;
			match data_mode {
				DataMode::ActiveMemory0 => {
					let expression = self.parse_instructions()?;
					let segment_size = leb128::read::unsigned(&mut self.bytecode)? as usize;
					let mut segment_data = vec![0u8; segment_size];
					self.bytecode.read_exact(&mut segment_data)?;
					log::debug!("Expression:{:?} Segment data:{:?}", expression, String::from_utf8(segment_data));
				},
				DataMode::Passive => unimplemented!(),
				DataMode::Active => unimplemented!(),
			}
		}
		Ok(())
	}

	fn parse_custom_section(&mut self, section_size: u64) -> Result<(), ParsingError> {
		let mut sink = vec![0u8; section_size as usize];
		self.bytecode.read_exact(&mut sink)?;
		log::trace!("Skipping custom section with {} bytes", section_size);
		Ok(())
	}

	fn parse_module_internal(mut self) -> Result<Module, ParsingError> {
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
				SectionId::Data => self.parse_data_section()?,
				SectionId::Custom => self.parse_custom_section(section_size)?,
				other => {
					log::error!("Unknown section {:?}. Ending parsing with Ok", other);
					break
				},
			}
		}
		Ok(self.module)
	}
}