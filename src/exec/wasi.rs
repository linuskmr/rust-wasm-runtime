use std::{io, mem};
use std::borrow::Borrow;
use std::io::{IoSlice, Write};
use crate::exec::{ExecutionResult, FunctionSignature, Value};
use crate::exec::instance::InstanceRef;


pub(crate) fn fd_write(instance: &mut InstanceRef) -> ExecutionResult {
	let arg_result_ptr = instance.op_stack_pop_u32()? as usize;
	let arg_iovec_num = instance.op_stack_pop_u32()? as usize;
	let arg_iovec_ptr = instance.op_stack_pop_u32()? as usize;
	let arg_fd = instance.op_stack_pop_u32()?;
	log::debug!("wasi_snapshot_preview1.fd_write: result_ptr={:?}, iovec_len={:?} iovec_ptr={:?} fd={:?}", arg_result_ptr, arg_iovec_num, arg_iovec_ptr, arg_fd);

	let mem = instance.memory.as_mut().unwrap();

	let mut io_slices: Vec<IoSlice> = Vec::new();
	let mut iovec_ptr = arg_iovec_ptr;

	for _ in 0..arg_iovec_num {
		let iovec_addr = {
			let mut buf = [0u8; 4];
			buf.copy_from_slice(&mem.data[iovec_ptr..iovec_ptr+4]);
			u32::from_le_bytes(buf)
		} as usize;
		iovec_ptr += 4;

		let iovec_len = {
			let mut buf = [0u8; 4];
			buf.copy_from_slice(&mem.data[iovec_ptr..iovec_ptr+4]);
			u32::from_le_bytes(buf)
		} as usize;
		iovec_ptr += 4;

		let iovec_buf = &mem.data[iovec_addr..iovec_addr + iovec_len];
		iovec_ptr += iovec_len;

		io_slices.push(IoSlice::new(iovec_buf));
	}

	match io::stdout().write_vectored(&io_slices) {
		Ok(bytes_written) => {
			mem.data[arg_result_ptr..arg_result_ptr +4].copy_from_slice(&(bytes_written as u32).to_le_bytes()); // Bytes written
			instance.operand_stack.push(Value::I32(0)); // Errno: Success
		},
		Err(err) => {
			mem.data[arg_result_ptr..arg_result_ptr +4].copy_from_slice(&[0u8; 4]); // Bytes written: 0
			instance.operand_stack.push(Value::I32(err.raw_os_error().unwrap_or(-1) as i32)); // Errno
		},
	};

	Ok(())
}