use std::io;
use std::io::Write;
use crate::exec::{Callable, ExecutionResult, Instance};
use crate::parse::Module;

fn print(instance: &mut Instance) -> ExecutionResult {
	log::trace!("Called extern function 'print'");
	let len = instance.runtime.op_stack_pop()? as usize;
	let addr = instance.runtime.op_stack_pop()? as usize;
	let data = instance.runtime.mem_slice(addr..addr + len)?;
	io::stdout().write_all(data).unwrap();
	Ok(())
}

/// Adds wasi functions to a module's functions.
pub fn include(module: &mut Module) {
	module.functions.insert("print".to_string(), Callable::RustFunction(print));
}