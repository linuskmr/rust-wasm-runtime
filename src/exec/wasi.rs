use std::{io};

use std::io::{IoSlice, Write};
use crate::exec::{ExecutionResult, Value};
use crate::exec::instance::InstanceRef;


pub fn fd_write(instance: &mut InstanceRef) -> ExecutionResult {
	let result_ptr = instance.operand_stack.pop::<i32>()? as usize;
	let iovec_array_len = instance.operand_stack.pop::<i32>()? as usize;
	let iovec_array_ptr = instance.operand_stack.pop::<i32>()? as usize;
	let fd = instance.operand_stack.pop::<i32>()?;

	let mem = instance.memory.as_mut().unwrap();

	let mut io_slices: Vec<IoSlice> = Vec::new();

	let iovec_ptrs = {
		const IOVEC_SIZE: usize = 8;
		let start = iovec_array_ptr;
		let end = iovec_array_ptr + (iovec_array_len * IOVEC_SIZE);
		(start..end).step_by(IOVEC_SIZE)
	};

	for iovec_ptr in iovec_ptrs {
		let iovec_addr = mem.read::<u32>(iovec_ptr) as usize;
		let iovec_len = mem.read::<u32>(iovec_ptr + 4) as usize;
		let iovec_buf = &mem.data[iovec_addr..iovec_addr+iovec_len];
		io_slices.push(IoSlice::new(iovec_buf));
	}

	match io::stdout().write_vectored(&io_slices) {
		Ok(bytes_written) => {
			mem.data[result_ptr..result_ptr +4].copy_from_slice(&(bytes_written as u32).to_le_bytes()); // Bytes written
			instance.operand_stack.push(Value::I32(0)); // Errno: Success
		},
		Err(err) => {
			mem.data[result_ptr..result_ptr +4].copy_from_slice(&[0u8; 4]); // Bytes written: 0
			instance.operand_stack.push(Value::I32(err.raw_os_error().unwrap_or(-1) as i32)); // Errno
		},
	};

	Ok(())
}